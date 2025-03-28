use chrono::{DateTime, Local};
use sha2::{Digest, Sha256};

use std::ffi::OsString;
use std::fs::{DirEntry, File, Metadata};
use std::io::prelude::*;
use std::io::Write;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata of a file
#[derive(Clone, Debug)]
pub struct FileMeta {
    /// The size of the file in bytes
    len: u64,
    created: Option<SystemTime>,
    last_modified: Option<SystemTime>,
}

impl FileMeta {
    fn from_metadata(m: &Metadata) -> Self {
        let len = m.len();
        let created = m.created().ok();
        let last_modified = m.modified().ok();
        FileMeta {
            len,
            created,
            last_modified,
        }
    }
}

/// File information (as DirEntry), metadata, and SHA256 hash
/// also handles generating the CSV file
#[derive(Debug)]
pub struct FileInfo {
    e: DirEntry,
    meta: FileMeta,
    sha256: String,
}

impl FileInfo {
    /// Costly function to create a FileInfo from a DirEntry.
    /// sha256 hash is calculated here
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

    /// Gives the names of its parent and grandparent directories
    /// # Returns
    /// [Option<OsString>; 2] ([Parent, Grandparent])
    fn parent_and_parent_parent(&self) -> [Option<OsString>; 2] {
        let canp = self.e.path().canonicalize();
        if let Ok(p) = canp {
            let comps: Vec<_> = p.components().collect();
            let l = comps.len();
            // components because we only want the directory names
            // and not their path
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

    /// Returns the header for the CSV file
    pub fn header(sep: &str) -> String {
        [
            "rel_path",
            "file_name",
            "size",
            "created",
            "modified",
            "sha256",
            "parent_dir",
            "parent_parent",
            "full_path",
            "seen_from",
            "created_epoch",
            "modified_epoch",
        ]
        .map(|f| format!("\"{f}\""))
        .join(sep)
    }

    /// Given w: Write, writes the CSV line to it
    /// # Arguments
    /// * `paper` - (&mut W), writes here
    /// * `path_relative_to` -  the "rel_path" field is calculated relative to this path
    ///    (or the full path if None)
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
        // localtime
        let [created, modified] = [meta.created, meta.last_modified].map(|m| {
            m.map_or(String::from(""), |c| {
                let dt: DateTime<Local> = c.into();
                format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
            })
        });
        let [created_epoch, modified_epoch] = [meta.created, meta.last_modified].map(|m| {
            m.map_or(String::from(""), |c| {
                c.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
            })
        });
        let sha256 = self.sha256.clone();
        let full_path = self
            .full_path()
            .map(|v| v.conv_to_string())
            .unwrap_or_else(|e| format!("{e:?}"));
        let seen_from = if let Some(p) = path_relative_to {
            let p: &Path = p.as_ref();
            p.conv_to_string()
        } else {
            String::from("")
        };
        let mut out = String::from("\"");
        out.push_str(
            &[
                path,
                file_name,
                size,
                created,
                modified,
                sha256,
                parent_dir,
                parent_parent,
                full_path,
                seen_from,
                created_epoch,
                modified_epoch,
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
