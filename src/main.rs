#![allow(dead_code)]
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::fs::{DirEntry, File, Metadata};
use std::io;
use std::io::prelude::*;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn do_main() -> io::Result<()> {
    let mut w = BufWriter::new(io::stdout());
    for f in fs::read_dir(".")? {
        println!("{f:?}");
        let entry = f?;
        if entry.file_type()?.is_file() {
            let fil = FileInfo::from_entry(entry);
            println!("{:?}", fil);
            fil.unwrap().write_csvline(&mut w).unwrap();
        } else {
            println!("{:?}", entry.file_type());
            println!("{:?}", entry.metadata());
        }
    }
    w.flush().unwrap();
    println!("{}", FileInfo::header(","));
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

    fn file_name(&self) -> OsString {
        self.e.file_name()
    }

    fn path(&self) -> io::Result<PathBuf> {
        self.e.path().canonicalize()
    }

    fn parent_and_parent_parent(&self) -> [Option<OsString>; 2] {
        let p = self.e.path();
        let comps: Vec<_> = p.components().collect();
        let l = comps.len();
        [
            l.checked_sub(2)
                .and_then(|i| comps.get(i))
                .map(|&c| c.as_os_str().to_owned()),
            l.checked_sub(3)
                .and_then(|i| comps.get(i))
                .map(|&c| c.as_os_str().to_owned()),
        ]
    }

    fn header(sep: &str) -> String {
        [
            "file_name",
            "path",
            "parent_dir",
            "parent_parent",
            "size",
            "created",
            "modified",
            "accessed",
            "sha256",
        ]
        .join(sep)
    }

    fn write_csvline<W: Write>(&self, paper: &mut W) -> io::Result<()> {
        let file_name = self.file_name().conv_to_string();
        let path = self
            .path()
            .map(|v| v.conv_to_string())
            .unwrap_or_else(|e| format!("{e:?}"));
        let [p, pp] = self.parent_and_parent_parent();
        let parent_dir = p.map(|pat| pat.conv_to_string()).unwrap_or("".to_string());
        let parent_parent = pp.map(|pat| pat.conv_to_string()).unwrap_or("".to_string());
        let out = [file_name, path, parent_dir, parent_parent].join(",");
        writeln!(paper, "{out}")
    }
}

trait ConvToString {
    fn conv_to_string(self) -> String;
}

impl ConvToString for OsString {
    fn conv_to_string(self) -> String {
        self.to_string_lossy().to_string()
    }
}
impl ConvToString for PathBuf {
    fn conv_to_string(self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl ConvToString for String {
    fn conv_to_string(self) -> String {
        self
    }
}

fn main() {
    do_main().unwrap();
}
