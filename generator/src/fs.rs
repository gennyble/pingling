use std::{
    collections::HashMap,
    iter,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct Directory {
    pub base: PathBuf,
    pub directories: Vec<Directory>,
    pub files_by_extension: HashMap<String, Vec<PathBuf>>,
}

impl Directory {
    pub fn index<P: AsRef<Path>>(path: P) -> Result<Self, DirectoryError> {
        let path = path.as_ref().canonicalize()?;

        if !path.is_dir() {
            return Err(DirectoryError::NotADirectory(path.to_owned()));
        }

        let mut ret = Self {
            base: path.clone(),
            directories: vec![],
            files_by_extension: HashMap::new(),
        };

        for file in path.read_dir()? {
            let file = file?;
            let fpath = file.path();
            let ftype = file.file_type()?;

            if ftype.is_dir() && !fpath.ends_with(".git") {
                ret.directories.push(Directory::index(fpath)?);
            } else {
                let extension = fpath
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                match ret.files_by_extension.get_mut(&extension) {
                    Some(vec) => vec.push(fpath),
                    None => {
                        let vec = vec![fpath];
                        assert_eq!(None, ret.files_by_extension.insert(extension, vec));
                    }
                }
            }
        }

        Ok(ret)
    }

    pub fn clone_structure<P: AsRef<Path>, F>(
        &self,
        clone_to: P,
        should_symlink_file: F,
    ) -> Result<(), DirectoryError>
    where
        F: Fn(&Path, &Path) -> bool + Clone,
    {
        let clone_to = clone_to.as_ref().canonicalize()?;

        if !clone_to.is_dir() {
            return Err(DirectoryError::NotADirectory(clone_to.to_owned()));
        }

        for (_, files) in self.files_by_extension.iter() {
            for file in files {
                let base_relative_name = file.strip_prefix(&self.base).unwrap();
                let mut outpath = clone_to.clone();
                outpath.push(base_relative_name);

                if should_symlink_file(&file, &outpath) {
                    if !outpath.exists() {
                        std::os::unix::fs::symlink(file, outpath).unwrap();
                    }
                }
            }
        }

        for directory in &self.directories {
            let stem = directory.base.components().last().unwrap().as_os_str();

            let mut clone_to = clone_to.clone();
            clone_to.push(stem);

            if !clone_to.exists() {
                std::fs::create_dir(&clone_to)?;
            }
            directory.clone_structure(&clone_to, should_symlink_file.clone())?;
        }

        Ok(())
    }

    pub fn find_all_by_extension<S: AsRef<str>>(&self, ext: S) -> Vec<&Path> {
        let mut ret = match self.files_by_extension.get(ext.as_ref()) {
            Some(vec) => vec.iter().map(|pb| pb.as_path()).collect(),
            None => vec![],
        };

        for dir in &self.directories {
            ret.extend_from_slice(&dir.find_all_by_extension(ext.as_ref()));
        }

        ret
    }

    pub fn get_directory<P: AsRef<Path>>(&self, path: P) -> Option<&Directory> {
        let path = match path.as_ref().canonicalize().ok() {
            Some(s) => s,
            None => return None,
        };

        if &path == &self.base {
            return Some(self);
        }

        let unprefixed = path.strip_prefix(&self.base).ok().map(|stripped_path| {
            let first_component = match stripped_path.components().next() {
                Some(first) => first,
                None => return None,
            };

            for dir in &self.directories {
                match dir.base.components().last() {
                    Some(last) if last == first_component => {
                        return dir.get_directory(&path);
                    }
                    _ => (),
                }
            }

            return None;
        });

        unprefixed.flatten()
    }
}

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("{0} is not a directory")]
    NotADirectory(PathBuf),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}

//TODO: Maybe return a Result with an error type detailng why we failed.
// panic, maybe? Would it be a programming mistake.
pub fn relativise_path<A: AsRef<Path>, B: AsRef<Path>>(base: A, target: B) -> Option<PathBuf> {
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
