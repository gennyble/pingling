pub mod fs;

use std::{
    iter,
    path::{Path, PathBuf},
};

use parser::{
    element::{Block, Inline},
    Parser,
};

pub fn parse_file<P: AsRef<Path>>(path: P, files: &[&Path]) -> (String, String) {
    let txt = std::fs::read_to_string(path.as_ref()).unwrap();
    let mut parser: Parser = txt.parse().unwrap();

    // inlines_mut is broken and won't go to the full depth so this is a little
    // hacky workaround
    for inline in parser.inlines_mut() {
        make_interlinks(inline, files, path.as_ref());
    }

    let mut ret = String::new();

    let mut block_iter = parser.blocks.into_iter();
    let title = match block_iter.next() {
        Some(Block::Header { level, content }) if level == 1 => vec_inline_html(content),
        Some(block) => {
            ret.push_str(&block_html(block));
            String::new()
        }
        None => String::new(),
    };

    for block in block_iter {
        ret.push_str(&block_html(block));
    }

    (title, ret)
}

fn make_interlinks<P: AsRef<Path>>(inline: &mut Inline, files: &[&Path], path: P) {
    match inline {
        Inline::Italic { content } => {
            for inline in content {
                make_interlinks(inline, files, path.as_ref());
            }
        }
        Inline::Bold { content } => {
            for inline in content {
                make_interlinks(inline, files, path.as_ref());
            }
        }
        Inline::InterLink { location, name } => {
            println!("{} | {}", location, name);
            let found: Vec<&&Path> = files
                .iter()
                .filter(|p| p.ends_with(&format!("{}.md", location)))
                .collect();

            if found.len() != 1 {
                dbg!(location, found);
                panic!("Found files does not have a length of one!");
            }

            let mut relative = fs::relativise_path(path.as_ref(), found[0]).unwrap();
            relative.set_extension("html");

            *location = relative.to_string_lossy().to_string();
        }
        _ => (),
    }
}

fn block_html(block: Block) -> String {
    match block {
        Block::Header { level, content } => {
            format!(
                "<h{level}>{}</h{level}>\n",
                vec_inline_html(content),
                level = level
            )
        }
        Block::Paragraph { content } => format!("<p>{}</p>\n", vec_inline_html(content)),
        Block::CodeBlock { content, .. } => {
            format!("<pre><code>{}</pre></code>\n", html_escape(content))
        }
        Block::Image { src, alt } => format!("<img src=\"{}\" alt=\"{}\"/>\n", src, alt),
    }
}

fn vec_inline_html(vecinline: Vec<Inline>) -> String {
    let mut ret = String::new();

    for inline in vecinline {
        ret.push_str(&inline_html(inline));
    }

    ret
}

fn inline_html(inline: Inline) -> String {
    match inline {
        Inline::SoftBreak => String::from("<br>"),
        Inline::Text(txt) => html_escape(txt),
        Inline::Code(code) => format!("<code>{}</code>", html_escape(code)),
        Inline::Italic { content } => format!("<i>{}</i>", vec_inline_html(content)),
        Inline::Bold { content } => format!("<b>{}</b>", vec_inline_html(content)),
        Inline::InterLink { name, location } => {
            format!("<a href=\"{}\">{}</a>", location, name)
        }
        Inline::ReferenceLink { name, location } => {
            format!("<a href=\"{}\">{}</a>", location, name)
        }
        Inline::AbsoluteLink { name, location } => {
            let name = match name {
                Some(name) => name,
                None => location.clone(),
            };

            format!("<a href=\"{}\">{}</a>", location, name)
        }
    }
}

fn html_escape<S: AsRef<str>>(raw: S) -> String {
    raw.as_ref().replace("<", "&lt;").replace(">", "&gt;")
}
