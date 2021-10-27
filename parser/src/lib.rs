pub mod element;

use std::{collections::HashMap, fmt, str::FromStr};

use element::{Block, Inline};

use thiserror::Error;

pub struct Parser {
    pub blocks: Vec<Block>,
}

impl Parser {
    // parses into blocks and generates a map of link references
    fn first_pass<S: AsRef<str>>(
        raw: S,
    ) -> Result<(Vec<Block>, HashMap<String, String>), ParseError> {
        let mut blocks = vec![];
        let mut linkrefs = HashMap::new();

        let lines = raw.as_ref().lines();

        let mut last_line_paragraph = false;
        let mut code_block_active = false;
        for line in lines {
            // If a code block is active...
            if code_block_active {
                if line == "```" {
                    // Make sure we can find the end
                    code_block_active = false;
                    continue;
                } else {
                    // And add to the block
                    if let Some(Block::CodeBlock { content, .. }) = blocks.last_mut() {
                        if !line.is_empty() {
                            content.push_str(line);
                        }
                        content.push('\n');
                        continue;
                    } else {
                        // This shouldn't be possible
                        unreachable!("A code block is active but the last element was not a code block, how did this happen")
                    }
                }
            }

            // Filter out empty lines
            if line.is_empty() {
                last_line_paragraph = false;
                continue;
            }

            // We already passed where we find the end, so this is the beginning
            if line.starts_with("```") {
                last_line_paragraph = false;
                code_block_active = true;

                blocks.push(Block::CodeBlock {
                    language: line.strip_prefix("```").unwrap().to_owned(),
                    content: String::new(),
                });

                continue;
            }

            // Lines starts `[`, assume it's the link for a reference...
            if line.starts_with('[') {
                match line.split_once("]: ") {
                    // But don't just unwrap, check, too
                    None => (),
                    Some((ref_name, ref_link)) => {
                        last_line_paragraph = false;
                        linkrefs.insert(ref_name[1..].to_owned(), ref_link.to_owned());
                        continue;
                    }
                }
            }

            // Find headers and make sure they're headers of 4 or less levels
            if line.starts_with("#") {
                match line.split_once(" ") {
                    Some((header, text)) => {
                        if header.replacen('#', "", 4).is_empty() {
                            last_line_paragraph = false;

                            blocks.push(Block::Header {
                                level: header.len() as u8,
                                content: vec![Inline::Text(text.to_owned())],
                            });
                            continue;
                        }
                    }
                    _ => (),
                }
            }

            // If we hit nothing else, we're a paragraph
            if !last_line_paragraph {
                blocks.push(Block::Paragraph {
                    content: vec![Inline::Text(line.to_owned())],
                });
            } else {
                if let Some(Block::Paragraph { content }) = blocks.last() {
                    match line.strip_prefix("^") {
                        Some(alt) => {
                            let link = if let Some(Block::Paragraph { content }) = blocks.last_mut()
                            {
                                match content.pop().unwrap() {
                                    Inline::Text(txt) => txt,
                                    _ => panic!(),
                                }
                            } else {
                                panic!()
                            };

                            blocks.push(Block::Image {
                                src: link,
                                alt: alt.trim_start().to_owned(),
                            })
                        }
                        None => {
                            if let Some(Block::Paragraph { content }) = blocks.last_mut() {
                                content.push(Inline::SoftBreak);
                                content.push(Inline::Text(line.to_owned()));
                            }
                        }
                    }
                } else {
                    // This shouldn't be able to happen
                    unreachable!(
                        "last_line_was_paragraph but the last line was not, what's gone wrong?"
                    )
                }
            }
            last_line_paragraph = true;
        }

        Ok((blocks, linkrefs))
    }

    fn second_pass(
        mut blocks: Vec<Block>,
        linkrefs: HashMap<String, String>,
    ) -> Result<Vec<Block>, ParseError> {
        for block in blocks.iter_mut() {
            match block {
                Block::Header { content, .. } => {
                    *content = Parser::parse_inlines(content, &linkrefs)
                }
                Block::Paragraph { content } => {
                    *content = Parser::parse_inlines(content, &linkrefs)
                }
                Block::Image { src, .. } => {
                    let link_inline = &Parser::do_links(&src, &linkrefs)[1];
                    let location = match link_inline {
                        Inline::InterLink { location, .. } => location,
                        Inline::ReferenceLink { location, .. } => location,
                        _ => panic!(),
                    };
                    *src = location.clone();
                }
                _ => (),
            }
        }

        Ok(blocks)
    }

    fn parse_inlines(inlines: &Vec<Inline>, linkrefs: &HashMap<String, String>) -> Vec<Inline> {
        let mut ret = vec![];

        for inline in inlines {
            match inline {
                Inline::SoftBreak => ret.push(Inline::SoftBreak),
                Inline::Text(txt) => ret.extend_from_slice(&Parser::parse_inline(txt, &linkrefs)),
                _ => (),
            }
        }

        ret
    }

    fn parse_inline<S: AsRef<str>>(raw: S, linkrefs: &HashMap<String, String>) -> Vec<Inline> {
        let raw = raw.as_ref();

        let mut tokens = vec![];
        let mut current = String::new();

        // Special flag for code
        let mut code_active = false;

        let mut chars = raw.chars().peekable();
        loop {
            match chars.next() {
                Some('`') => {
                    if code_active {
                        match tokens.last_mut() {
                            Some(Token::Code(st)) => *st = current.clone(),
                            _ => unreachable!(),
                        }
                        current.clear();
                        code_active = false
                    } else {
                        tokens.push(Token::Text(current.clone()));
                        tokens.push(Token::Code(String::new()));
                        current.clear();
                        code_active = true
                    }
                }
                Some(ch) if code_active => current.push(ch),
                // Other patterns
                Some('*') => {
                    match chars.peek() {
                        // Bold
                        Some('*') => {
                            chars.next(); // take the one we just peeked
                            tokens.push(Token::Text(current.clone()));
                            tokens.push(Token::Bold);
                            current.clear();
                        }
                        // Italic
                        _ => {
                            tokens.push(Token::Text(current.clone()));
                            tokens.push(Token::Italic);
                            current.clear();
                        }
                    }
                }
                // Default
                Some(ch) => current.push(ch),
                None => {
                    tokens.push(Token::Text(current));
                    break;
                }
            }
        }

        let mut inlines = vec![];
        let mut stack: Vec<TokenOrInline> = vec![];

        for token in tokens {
            match token {
                Token::Text(txt) => {
                    let ils = Parser::do_links(txt, linkrefs);
                    if stack.is_empty() {
                        inlines.extend_from_slice(&ils);
                    } else {
                        let toi = ils
                            .into_iter()
                            .map(|il| TokenOrInline::Inline(il))
                            .collect::<Vec<TokenOrInline>>();
                        stack.extend_from_slice(&toi);
                    }
                }
                Token::Code(cd) => {
                    if stack.is_empty() {
                        inlines.push(Inline::Code(cd))
                    } else {
                        stack.push(Inline::Code(cd).into())
                    }
                }
                Token::Italic => {
                    if stack.contains(&Token::Italic.into()) {
                        let tokens = Parser::pop_until(&mut stack, Token::Italic.into());
                        stack.push(
                            Inline::Italic {
                                content: TokenOrInline::vec_inlines(tokens),
                            }
                            .into(),
                        )
                    } else {
                        stack.push(Token::Italic.into())
                    }
                }
                Token::Bold => {
                    if stack.contains(&Token::Bold.into()) {
                        let tokens = Parser::pop_until(&mut stack, Token::Bold.into());
                        stack.push(
                            Inline::Bold {
                                content: TokenOrInline::vec_inlines(tokens),
                            }
                            .into(),
                        )
                    } else {
                        stack.push(Token::Bold.into())
                    }
                }
            }

            if stack.len() == 1 {
                match stack.pop() {
                    Some(TokenOrInline::Inline(il)) => inlines.push(il),
                    Some(toi) => stack.push(toi),
                    None => (),
                }
            }
        }

        inlines
    }

    fn do_links<S: AsRef<str>>(raw: S, linkrefs: &HashMap<String, String>) -> Vec<Inline> {
        let raw = raw.as_ref();

        // Find absolute links.
        // NOTE: There still may be reference or interlinks before this!
        match raw.find("{{") {
            Some(start) => match raw.find("}}") {
                Some(end) => {
                    let before = &raw[..start];
                    let link = &raw[start + 2..end];
                    let after = &raw[end + 2..];

                    let mut inlines = Self::do_links(before, linkrefs);
                    inlines.push(Inline::AbsoluteLink {
                        location: link.to_owned(),
                    });
                    inlines.extend_from_slice(&Self::do_links(after, linkrefs));

                    return inlines;
                }
                None => (),
            },
            None => (),
        }

        // Find reference and interlinks.
        match raw.find("{") {
            Some(start) => match raw.find("}") {
                Some(end) => {
                    let before = Inline::Text(raw[..start].to_owned());
                    let link = &raw[start + 1..end];
                    let after = &raw[end + 1..];

                    let mut inlines = vec![before, Self::get_link(link, linkrefs)];
                    inlines.extend_from_slice(&Self::do_links(after, linkrefs));

                    return inlines;
                }
                None => (),
            },
            None => (),
        }

        vec![Inline::Text(raw.to_owned())]
    }

    fn get_link<S: AsRef<str>>(raw: S, linkrefs: &HashMap<String, String>) -> Inline {
        let raw = raw.as_ref();

        match raw.chars().next() {
            // Reference link!
            Some('!') => match linkrefs.get(&raw[1..]) {
                Some(location) => Inline::ReferenceLink {
                    name: raw[1..].to_owned(),
                    location: location.to_owned(),
                },
                None => {
                    // Recover when we can't find the location by reconstructing the text
                    Inline::Text(format!("{{{}}}", raw))
                }
            },
            // Interlink!
            Some(_) => Inline::InterLink {
                name: raw.to_owned(),
                location: String::new(),
            },
            None => {
                // We're a link, but we're an empty link. Give back an Inline text with only {}
                Inline::Text("{}".to_owned())
            }
        }
    }

    fn pop_until<T: PartialEq>(haystack: &mut Vec<T>, needle: T) -> Vec<T> {
        let mut ret = vec![];
        loop {
            match haystack.pop() {
                Some(thing) if thing == needle => break,
                Some(thing) => ret.push(thing),
                None => break,
            }
        }

        ret.reverse();
        ret
    }

    pub fn inlines_mut<'a>(&'a mut self) -> InlineIter<'a> {
        InlineIter::new(self)
    }
}

pub struct InlineIter<'a> {
    blocks: Option<std::slice::IterMut<'a, Block>>,
    inlines: Option<std::slice::IterMut<'a, Inline>>,
}

impl<'a> InlineIter<'a> {
    fn new(parser: &'a mut Parser) -> Self {
        let blocks = parser.blocks.iter_mut();

        Self {
            blocks: Some(blocks),
            inlines: None,
        }
    }
}

impl<'a> Iterator for InlineIter<'a> {
    type Item = &'a mut Inline;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut inline_iter) = self.inlines {
            match inline_iter.next() {
                Some(inline) => return Some(inline),
                None => (),
            }
        }

        if let Some(ref mut block_iter) = self.blocks {
            loop {
                match block_iter.next() {
                    None => {
                        self.blocks = None;
                        return None;
                    }
                    Some(block) => match block {
                        Block::Header { content, .. } => {
                            self.inlines = Some(content.iter_mut());
                            break;
                        }
                        Block::Paragraph { content } => {
                            self.inlines = Some(content.iter_mut());
                            break;
                        }
                        Block::CodeBlock { language, content } => continue,
                        Block::Image { src, alt } => continue,
                    },
                }
            }

            self.next()
        } else {
            None
        }
    }
}

impl FromStr for Parser {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (blocks, linkrefs) = Self::first_pass(s)?;
        let blocks = Self::second_pass(blocks, linkrefs)?;

        Ok(Self { blocks })
    }
}

#[derive(Debug, Error)]
pub enum ParseError {}

#[derive(Clone, PartialEq)]
enum TokenOrInline {
    Token(Token),
    Inline(Inline),
}

impl TokenOrInline {
    fn vec_inlines(vec: Vec<Self>) -> Vec<Inline> {
        let mut ret = vec![];

        for v in vec {
            if let TokenOrInline::Inline(inline) = v {
                ret.push(inline);
            } else {
                panic!("Token in vec_inlines")
            }
        }

        ret
    }

    fn is_token(&self) -> bool {
        match self {
            TokenOrInline::Token(_) => true,
            _ => false,
        }
    }
}

impl From<Token> for TokenOrInline {
    fn from(t: Token) -> Self {
        TokenOrInline::Token(t)
    }
}

impl From<Inline> for TokenOrInline {
    fn from(t: Inline) -> Self {
        TokenOrInline::Inline(t)
    }
}

#[derive(Clone, PartialEq)]
enum Token {
    Text(String),
    Code(String),
    Italic,
    Bold,
}

impl Token {
    fn vec_string(tokens: Vec<Token>) -> String {
        let mut ret = String::new();
        for token in tokens {
            ret.push_str(&token.to_string());
        }

        ret
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Text(txt) => write!(f, "{}", txt),
            Token::Code(st) => write!(f, "`{}`", st),
            Token::Italic => write!(f, "*"),
            Token::Bold => write!(f, "**"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
