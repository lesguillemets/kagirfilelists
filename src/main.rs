use std::fs;
use std::fs::{DirEntry, File};
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use rayon::iter::Either;
use rayon::prelude::*;

use kagirfilelists::FileInfo;

#[derive(Parser, Debug)]
struct Cli {
    /// The directory to traverse
    #[arg(value_name = "DIR", help = "The directory to traverse")]
    path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "The file to which the output should be written"
    )]
    output: Option<PathBuf>,
    #[arg(short, long, help = "Overwrites existing file if specified")]
    force: bool,
    #[arg(long, help = "Adds a BOM in front of the file")]
    with_bom: bool,
    #[arg(short, long, help = "More verbose in logging")]
    verbose: bool,
}

impl Cli {
    /// basically the main function
    fn try_main(&self) -> Result<(), Vec<(PathBuf, io::Error)>> {
        if let Some(f) = &self.output {
            // A file is specified
            let exists = f.try_exists();
            if exists.is_ok() && !exists.unwrap() {
                // confirmed not to exist
                let file = File::create(f).expect("unable to create file!");
                let mut w = BufWriter::new(file);
                if self.with_bom {
                    w.write_all(&[0xef, 0xbb, 0xbf]).expect("error writing bom");
                }
                eprintln!("Starting");
                self.try_run(&mut w)
            } else {
                // specified an existing file
                eprintln!("Warn: File exists: {f:?}");
                unimplemented!(
                    "-f or --force is intended to allow overwriting, but still unimplemented"
                );
            }
        } else {
            // write to stdout
            let mut w = BufWriter::new(io::stdout());
            self.try_run(&mut w)
        }
    }

    /// Called by try_main, generates csv and writes to w
    fn try_run<W: Write>(&self, w: &mut W) -> Result<(), Vec<(PathBuf, io::Error)>> {
        // early return if you encounter errors in writing the header
        if let Err(header_error) = writeln!(w, "{}", FileInfo::header(",")) {
            return Err(vec![("".into(), header_error)]);
        }
        let dir = self.get_path();
        let errors = self.read_dir(w, &dir);
        eprintln!("finished");
        w.flush().unwrap();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Recursively reads the directory. Files, then directories
    fn read_dir<T: Write>(&self, w: &mut T, dir: &PathBuf) -> Vec<(PathBuf, io::Error)> {
        let mut the_errors: Vec<(PathBuf, io::Error)> = vec![];
        if self.verbose {
            eprintln!(">> Info: reading directory {:?}", &dir);
        } else {
            eprint!("\x1b[2K\r>> Info: reading directory {:?}", &dir);
        }
        let (entries, failed): (Vec<_>, Vec<_>) = fs::read_dir(dir)
            .unwrap_or_else(|_| panic!("error in fs::read_dir{dir:?}"))
            .partition(|e| e.is_ok());
        // not sure when this happens, so just print it
        for fail in &failed {
            eprintln!("ERROR: {fail:?}");
        }
        // partition files and directories
        let (files, others): (Vec<DirEntry>, Vec<DirEntry>) =
            entries.into_iter().map(|e| e.unwrap()).partition(|e| {
                e.file_type()
                    .unwrap_or_else(|_| panic!("Cli::read_dir,file_type on {e:?}"))
                    .is_file()
            });
        // For files, just add to ther result
        let (oks, mut errs): (Vec<Vec<u8>>, Vec<(PathBuf, io::Error)>) =
            files.into_par_iter().partition_map(|f| {
                let path = f.path();
                if self.verbose {
                    eprint!("\x1b[2K\r");
                    eprint!(":: >> Info: reading file {:?}", &path);
                }
                let result = self.report_file_as_csv(f);
                if let Ok(res) = result {
                    Either::Left(res)
                } else {
                    Either::Right((path, result.unwrap_err()))
                }
            });
        // report errors
        for (err_entry, er) in &errs {
            eprintln!("ERROR in file {err_entry:?} for write_csvline, {er:?}");
        }
        // and save them for later
        the_errors.append(&mut errs);
        // write resulting ok csv lines
        let lines: Vec<u8> = oks.into_iter().flatten().collect();
        w.write_all(&lines).unwrap();
        // For directories, go deeper
        for other in &others {
            let ft = other
                .file_type()
                .unwrap_or_else(|_| panic!("error in DirEntry.file_type() for {other:?}"));
            if ft.is_dir() {
                // go deeper
                let mut direrrors = self.read_dir(w, &other.path());
                the_errors.append(&mut direrrors);
            } else {
                // Neither file nor directory. symlinks?
                eprintln!("WARN:::: {:?}", other.file_type());
                eprintln!("    :::: {:?}", other.metadata());
                eprintln!("    :::: {:?}", other);
            }
        }
        the_errors
    }

    /// read a file and report resulting csv as writable bytes
    fn report_file_as_csv(&self, f: DirEntry) -> io::Result<Vec<u8>> {
        let mut s: Vec<u8> = Vec::new();
        let fil = FileInfo::from_entry(f)?;
        fil.write_csvline(&mut s, &Some(self.get_path()))?;
        Ok(s)
    }

    /// returns the specified dir (if none, '.')
    fn get_path(&self) -> PathBuf {
        self.path
            .clone()
            .unwrap_or(".".into())
            .canonicalize()
            .unwrap()
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");
    cli.try_main().unwrap();
}
