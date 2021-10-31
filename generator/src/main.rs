use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use bempline::{Document, Options};
use confindent::Confindent;
use generator::{fs::Directory, parse_file};

fn main() {
    let conf = match Confindent::from_file("generator.conf") {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("Could not parse the configuration file: {}", e);
            std::process::exit(-1);
        }
    };

    let root = match conf.child_value("Root") {
        Some(val) => PathBuf::from(val),
        None => {
            eprintln!(
                "Please specify where the wikarden root is in the coniguration with the `Root` key"
            );
            std::process::exit(-1);
        }
    };

    let target = match conf.child_value("Target") {
        Some(val) => PathBuf::from(val),
        None => {
            eprintln!(
                "Please specify where the HTML wikarden will be placed with the configuration `Target` key"
            );
            std::process::exit(-1);
        }
    };

    let canon = match root.canonicalize() {
        Ok(canoned) => canoned,
        Err(e) => {
            eprintln!("Could not convert root path to absolute: {}", e);
            std::process::exit(-1);
        }
    };

    let target_canon = match target.canonicalize() {
        Ok(canoned) => canoned,
        Err(e) => {
            eprintln!("Could not convert target path to absolute: {}", e);
            std::process::exit(-1);
        }
    };

    let doc =
        Document::from_file(conf.child_value("Template").unwrap(), Options::default()).unwrap();

    let root_directory = Directory::index(canon).unwrap();
    let mds = root_directory.find_all_by_extension("md");

    root_directory
        .clone_structure(target_canon, |from, to| match from.extension() {
            Some(ext) => {
                if ext.to_string_lossy() == "md" {
                    let mut to = to.to_owned();
                    to.set_extension("html");

                    let mut from_no_ext = from.to_owned();
                    from_no_ext.set_extension("");

                    let file_stem = to.file_stem().unwrap().to_str().unwrap();

                    let mut from_dir = from.to_owned();
                    from_dir.pop();

                    let mut doc = doc.clone();
                    doc.set("page_title", to.file_stem().unwrap().to_string_lossy());
                    doc.set("title", to.file_stem().unwrap().to_string_lossy());

                    match find_triplet(&root_directory, from) {
                        ((_, None), (current_bit, Some(current)), None) => {
                            doc.set("parents", make_current(current_bit, current));
                            doc.set("current", "");
                            doc.set("children", "");
                        }
                        ((_, None), (current_bit, Some(current)), Some(children)) => {
                            doc.set("parents", make_current(current_bit, current));
                            doc.set("current", make_children(file_stem, children));
                            doc.set("children", "");
                        }
                        ((parent_bit, Some(parent)), (current_bit, Some(current)), None) => {
                            doc.set("parents", make_parent(parent_bit, parent));
                            doc.set("current", make_current(current_bit, current));
                            doc.set("children", "");
                        }
                        (
                            (parent_bit, Some(parent)),
                            (current_bit, Some(current)),
                            Some(children),
                        ) => {
                            doc.set("parents", make_parent(parent_bit, parent));
                            doc.set("current", make_current(current_bit, current));
                            doc.set("children", make_children(file_stem, children));
                        }
                        _ => unreachable!(),
                    };

                    let parsed = parse_file(from, &mds, &root_directory);
                    doc.set("body", parsed);

                    let mut file = File::create(to).unwrap();
                    file.write_all(doc.compile().as_bytes()).unwrap();

                    false
                } else {
                    true
                }
            }
            None => true,
        })
        .unwrap();
}

fn find_triplet<'r>(
    root: &'r Directory,
    from: &Path,
) -> (
    (String, Option<&'r Directory>),
    (String, Option<&'r Directory>),
    Option<&'r Directory>,
) {
    let mut search = from.to_owned();
    search.set_extension("");

    // Check if there's a folder with our filename. If there is, make children
    let children = root.get_directory(&search);
    let children_last = search
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();

    search.pop();
    // This should never be none. Should we check that?
    let current = root.get_directory(&search);
    let current_last = search
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();

    search.pop();
    let parent = root.get_directory(&search);
    let parent_last = search
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();

    ((current_last, parent), (children_last, current), children)
}

fn get_paths(dir: &Directory) -> Vec<PathBuf> {
    let mut paths = vec![];

    for files in dir.files_by_extension.get("md") {
        for file in files {
            let mut no_ext = file.clone();
            no_ext.set_extension("");
            paths.push(no_ext.strip_prefix(&dir.base).unwrap().to_owned());
        }
    }

    paths
}

fn make_parent(parent_bit: String, dir: &Directory) -> String {
    let mut ret = String::new();

    for path in get_paths(dir) {
        let pathstr = path.to_string_lossy().to_string();
        if parent_bit == pathstr {
            ret.push_str(&format!(
                "<a id=\"current-directory\" href=\"../{name}.html\">{name}</a>",
                name = path.to_string_lossy()
            ));
        } else {
            ret.push_str(&format!(
                "<a href=\"../{name}.html\">{name}</a>",
                name = path.to_string_lossy()
            ));
        }
    }

    ret
}

fn make_current(current_bit: String, dir: &Directory) -> String {
    let mut ret = String::new();

    for path in get_paths(dir) {
        let pathstr = path.to_string_lossy().to_string();
        if current_bit == pathstr {
            ret.push_str(&format!(
                "<a id=\"current-file\" href=\"{name}.html\">{name}</a>",
                name = path.to_string_lossy()
            ));
        } else {
            ret.push_str(&format!(
                "<a href=\"{name}.html\">{name}</a>",
                name = path.to_string_lossy()
            ));
        }
    }

    ret
}

fn make_children(current: &str, dir: &Directory) -> String {
    let mut ret = String::new();

    for path in get_paths(dir) {
        ret.push_str(&format!(
            "<a href=\"{}/{name}.html\">{name}</a>",
            current,
            name = path.to_string_lossy()
        ));
    }

    ret
}
