use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use confindent::Confindent;
use generator::parse_file;
use thiserror::Error;

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

    println!("Wikarden root: {}", canon.to_string_lossy());

    let index = index_directory(&canon).unwrap();
    for (is_dir, path) in &index {
        println!("[{}] {}", is_dir, path.to_string_lossy())
    }

    // Do we need the directory? Let me just grab the files..
    let files: Vec<&Path> = index
        .iter()
        .filter_map(|(is_dir, path)| if !is_dir { Some(path.as_path()) } else { None })
        .collect();

    let directories: Vec<&Path> = index
        .iter()
        .filter_map(|(is_dir, path)| if *is_dir { Some(path.as_path()) } else { None })
        .collect();

    println!("FILES:");
    for file in &files {
        println!("{}", file.to_string_lossy());
    }

    // Go through and make all the required directories
    for dir in directories {
        let dirname = dir.strip_prefix(&canon).unwrap();
        let mut dir = target_canon.clone();
        dir.push(dirname);
        fs::create_dir_all(dir).unwrap();
    }

    for file in files {
        let filename = file.strip_prefix(&canon).unwrap();
        let mut outfile = target_canon.clone();
        outfile.push(filename);
        outfile.set_extension("html");

        let parsed = format!("<html><body>{}</body></html>", parse_file(file));
        let mut file = fs::File::create(outfile).unwrap();
        file.write_all(parsed.as_bytes()).unwrap();
    }
}

fn index_directory<P: AsRef<Path>>(search_path: P) -> Result<Vec<(bool, PathBuf)>, IndexError> {
    let search_path = search_path.as_ref();
    let mut ret = vec![];

    if !search_path.is_dir() {
        return Err(IndexError::NotADirectory(search_path.to_owned()));
    }

    for file in search_path.read_dir()? {
        let file = file?;
        let fpath = file.path();
        let ftype = file.file_type()?;

        if ftype.is_dir() {
            ret.extend_from_slice(&index_directory(&fpath)?);
            ret.push((true, fpath));
        } else {
            ret.push((false, fpath));
        }
    }

    Ok(ret)
}

#[derive(Debug, Error)]
enum IndexError {
    #[error("{0} is not a directory")]
    NotADirectory(PathBuf),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
