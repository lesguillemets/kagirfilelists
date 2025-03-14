use sha2::{Digest, Sha256};
use std::fs;
use std::fs::{DirEntry, File};
use std::io;
use std::io::prelude::*;

fn do_main() -> io::Result<()> {
    for f in fs::read_dir(".")? {
        println!("{f:?}");
        let entry = f?;
        println!("{:?}", entry.file_type());
        println!("{:?}", entry.metadata());
        if entry.file_type()?.is_file() {
            let mut f = File::open(entry.path())?;
            let mut hasher = Sha256::new();
            let mut b = Vec::new();
            f.read_to_end(&mut b)?;
            hasher.write_all(&b)?;
            let result = hasher.finalize();
            println! {"{:x}", result};
        }
    }
    Ok(())
}

#[derive(Debug)]
struct FileEntry {
    e: DirEntry,
}

impl FileEntry {
    fn from_entry(e: DirEntry) -> io::Result<Self> {
        if e.file_type()?.is_file() {
            Ok(FileEntry { e })
        } else if e.file_type()?.is_dir() {
            Err(io::Error::new(
                io::ErrorKind::IsADirectory,
                "Is a directory",
            ))
        } else {
            Err(io::Error::other("Not a file or directory"))
        }
    }
}

fn main() {
    do_main().unwrap();
}
