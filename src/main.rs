use sha2::{Digest, Sha256};
use std::fs;
use std::fs::{DirEntry, File, Metadata};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

fn do_main() -> io::Result<()> {
    for f in fs::read_dir(".")? {
        println!("{f:?}");
        let entry = f?;
        if entry.file_type()?.is_file() {
            let fil = FileInfo::from_entry(entry);
            println!("{:?}", fil);
        } else {
            println!("{:?}", entry.file_type());
            println!("{:?}", entry.metadata());
        }
    }
    Ok(())
}

fn calc_sha256<P: AsRef<Path>>(p: P) -> io::Result<String> {
    let mut f = File::open(p)?;
    let mut hasher = Sha256::new();
    let mut b = Vec::new();
    f.read_to_end(&mut b)?;
    hasher.write_all(&b)?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Clone, Debug)]
struct FileMeta {
    len: u64,
    created: Option<SystemTime>,
    last_modified: Option<SystemTime>,
    last_accessed: Option<SystemTime>,
}

impl FileMeta {
    fn from_metadata(m: &Metadata) -> Self {
        let len = m.len();
        let created = m.created().ok();
        let last_modified = m.modified().ok();
        let last_accessed = m.accessed().ok();
        FileMeta {
            len,
            created,
            last_modified,
            last_accessed,
        }
    }
}

#[derive(Debug)]
struct FileInfo {
    e: DirEntry,
    meta: FileMeta,
    sha256: String,
}

impl FileInfo {
    fn from_entry(e: DirEntry) -> io::Result<Self> {
        if e.file_type()?.is_file() {
            let meta = FileMeta::from_metadata(&e.metadata()?);
            let sha256 = calc_sha256(e.path())?;
            Ok(FileInfo { e, meta, sha256 })
        } else if e.file_type()?.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::IsADirectory,
                "Is a directory",
            ))
        } else {
            Err(io::Error::other("Not a file or directory"))
        }
    }

    fn write_to_csvline<W: Write>(&self, paper: &mut W) -> io::Result<()> {
        write!(paper, "")
    }
}

fn main() {
    do_main().unwrap();
}
