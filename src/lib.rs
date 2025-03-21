use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs::{DirEntry, File, Metadata};
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct FileMeta {
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
pub struct FileInfo {
    e: DirEntry,
    meta: FileMeta,
    sha256: String,
}

impl FileInfo {
    pub fn from_entry(e: DirEntry) -> io::Result<Self> {
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

    fn full_path(&self) -> io::Result<PathBuf> {
        self.e.path().canonicalize()
    }

    fn parent_and_parent_parent(&self) -> [Option<OsString>; 2] {
        let canp = self.e.path().canonicalize();
        if let Ok(p) = canp {
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
        } else {
            [None, None]
        }
    }

    pub fn header(sep: &str) -> String {
        [
            "file_name",
            "rel_path",
            "parent_dir",
            "parent_parent",
            "size",
            "created",
            "modified",
            "accessed",
            "sha256",
            "full_path",
        ]
        .map(|f| format!("\"{f}\""))
        .join(sep)
    }

    pub fn write_csvline<W: Write, T: AsRef<Path>>(
        &self,
        paper: &mut W,
        path_relative_to: &Option<T>,
    ) -> io::Result<()> {
        // TODO: we don't really handle double quotes in fields
        let file_name = self.file_name().conv_to_string();
        let path = self.e.path();
        let path = if let Some(p) = path_relative_to {
            path.strip_prefix(p).unwrap_or(&path)
        } else {
            &path
        }
        .conv_to_string();
        let [p, pp] = self.parent_and_parent_parent();
        let parent_dir = p.map(|pat| pat.conv_to_string()).unwrap_or("".to_string());
        let parent_parent = pp.map(|pat| pat.conv_to_string()).unwrap_or("".to_string());
        let size = self.meta.len.to_string();
        let meta = &self.meta;
        let [created, modified, accessed] = [meta.created, meta.last_modified, meta.last_accessed]
            .map(|m| {
                m.map_or(String::from(""), |c| {
                    c.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
                })
            });
        let sha256 = self.sha256.clone();
        let full_path = self
            .full_path()
            .map(|v| v.conv_to_string())
            .unwrap_or_else(|e| format!("{e:?}"));
        let mut out = String::from("\"");
        out.push_str(
            &[
                file_name,
                path,
                parent_dir,
                parent_parent,
                size,
                created,
                modified,
                accessed,
                sha256,
                full_path,
            ]
            .join("\",\""),
        );
        out.push('\"');
        writeln!(paper, "{out}")
    }
}

const SHA_BUFSIZE: usize = 65536;

fn calc_sha256<P: AsRef<Path>>(p: P) -> io::Result<String> {
    let f = File::open(p)?;
    let mut reader = BufReader::new(f);
    let mut buffer = [0; SHA_BUFSIZE];
    let mut hasher = Sha256::new();
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub trait ConvToString {
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
impl ConvToString for &Path {
    fn conv_to_string(self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl ConvToString for String {
    fn conv_to_string(self) -> String {
        self
    }
}
