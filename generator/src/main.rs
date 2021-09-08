use std::path::{Path, PathBuf};

use confindent::Confindent;
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

    let canon = match root.canonicalize() {
        Ok(canoned) => canoned,
        Err(e) => {
            eprintln!("Could not convert root path to absolute: {}", e);
            std::process::exit(-1);
        }
    };

    println!("Wikarden root: {}", canon.to_string_lossy());

    let files = index_directory(&canon).unwrap();
    for (is_dir, path) in files {
        println!("[{}] {}", is_dir, path.to_string_lossy())
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
