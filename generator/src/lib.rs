use std::{
    iter,
    path::{Path, PathBuf},
};

use parser::{
    element::{Block, Inline},
    Parser,
};

pub fn parse_file<P: AsRef<Path>>(path: P, files: &[&Path]) -> String {
    let txt = std::fs::read_to_string(path.as_ref()).unwrap();
    let mut parser: Parser = txt.parse().unwrap();

    for inline in parser.inlines_mut() {
        match inline {
            Inline::InterLink { name, location } => {
                let found: Vec<&&Path> = files
                    .iter()
                    .filter(|p| p.ends_with(&format!("{}.md", name)))
                    .collect();

                if found.len() != 1 {
                    panic!("Found files does not have a length of one!");
                }

                let mut relative = relativise_path(path.as_ref(), found[0]).unwrap();
                relative.set_extension("html");

                *location = relative.to_string_lossy().to_string();
            }
            _ => (),
        }
    }

    let mut ret = String::new();
    for block in parser.blocks {
        ret.push_str(&block_html(block));
    }

    ret
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
        Inline::AbsoluteLink { location } => {
            format!("<a href=\"{location}\">{location}</a>", location = location)
        }
    }
}

fn html_escape<S: AsRef<str>>(raw: S) -> String {
    raw.as_ref().replace("<", "&lt;").replace(">", "&gt;")
}

//TODO: Maybe return a Result with an error type detailng why we failed.
// panic, maybe? Would it be a programming mistake.
fn relativise_path<A: AsRef<Path>, B: AsRef<Path>>(base: A, target: B) -> Option<PathBuf> {
    let mut base = base.as_ref().to_owned();
    let target = target.as_ref().to_owned();

    if base.is_relative() || target.is_relative() {
        // We need both to be absolute
        return None;
    }

    if base.is_file() {
        if !base.pop() {
            // base was previously known to be absolute, but we popped and there
            // wasn't a parent. How can that happen?
            return None;
        }
    }

    let mut pop_count = 0;
    loop {
        if target.starts_with(&base) {
            break;
        }

        if !base.pop() {
            // We're at the root, done.
            break;
        } else {
            pop_count += 1;
        }
    }

    let mut backtrack: PathBuf = iter::repeat("../").take(pop_count).collect();
    let target = target.strip_prefix(base).unwrap().to_owned();

    backtrack.push(target);
    Some(backtrack)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn relativise_path_same_dir() {
        let base = PathBuf::from("/srv/wikarden/");
        let targ = PathBuf::from("/srv/wikarden/testfile.md");

        assert_eq!(
            PathBuf::from("testfile.md"),
            relativise_path(&base, &targ).unwrap()
        )
    }

    #[test]
    fn relativise_path_below() {
        let base = PathBuf::from("/srv/wikarden/higher/tree");
        let targ = PathBuf::from("/srv/wikarden/testfile.md");

        assert_eq!(
            PathBuf::from("../../testfile.md"),
            relativise_path(&base, &targ).unwrap()
        )
    }

    #[test]
    fn relativise_path_above() {
        let base = PathBuf::from("/srv/wikarden/");
        let targ = PathBuf::from("/srv/wikarden/testdir/testfile.md");

        assert_eq!(
            PathBuf::from("testdir/testfile.md"),
            relativise_path(&base, &targ).unwrap()
        )
    }

    #[test]
    fn relativise_path_nothing() {
        let base = PathBuf::from("/opt/usr");
        let targ = PathBuf::from("/srv/testfile.md");

        assert_eq!(
            PathBuf::from("../../srv/testfile.md"),
            relativise_path(&base, &targ).unwrap()
        )
    }
}
