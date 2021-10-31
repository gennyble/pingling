use core::panic;

#[derive(Debug)]
pub enum Block {
    Header { level: u8, content: Vec<Inline> },
    Paragraph { content: Vec<Inline> },
    CodeBlock { language: String, content: String },
    Image { src: String, alt: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    SoftBreak,
    Text(String),
    Code(String),
    Italic { content: Vec<Inline> },
    Bold { content: Vec<Inline> },
    AbsoluteLink { location: String },
    ReferenceLink { name: String, location: String },
    InterLink { name: String, location: String },
}
