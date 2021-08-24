use std::fmt::format;

use parser::element::{Block, Inline};
use parser::Parser;

fn main() {
    let txt = std::fs::read_to_string("../test.md").unwrap();
    let parser: Parser = txt.parse().unwrap();

    let mut ret = String::from("<html><body>");

    for block in parser.blocks {
        ret.push_str(&block_html(block));
    }

    ret.push_str("</body></html>");
    print!("{}", ret);
}

fn block_html(block: Block) -> String {
    match block {
        Block::Header { level, content } => {
            format!(
                "<h{level}>{}</h{level}>",
                vec_inline_html(content),
                level = level
            )
        }
        Block::Paragraph { content } => format!("<p>{}</p>", vec_inline_html(content)),
        Block::CodeBlock { content, .. } => format!("<pre><code>{}</pre></code>", content),
        Block::Image { src, alt } => format!("<img src=\"{}\" alt=\"{}\"/>", src, alt),
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
        Inline::Text(txt) => txt,
        Inline::Code(code) => format!("<code>{}</code>", code),
        Inline::Italic { content } => format!("<i>{}</i>", vec_inline_html(content)),
        Inline::Bold { content } => format!("<b>{}</b>", vec_inline_html(content)),
        Inline::Link { location } => {
            format!("<a href=\"{location}\">{location}</a>", location = location)
        }
        Inline::ReferenceLink { name, location } => {
            format!("<a href=\"{}\">{}</a>", location, name)
        }
    }
}
