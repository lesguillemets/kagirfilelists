use std::fs;
use std::fs::{DirEntry, File};
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;

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
}

impl Cli {
    /// basically the main function
    fn try_main(&self) -> io::Result<()> {
        if let Some(f) = &self.output {
            // A file is specified
            let exists = f.try_exists();
            if exists.is_ok() && !exists.unwrap() {
                // confirmed not to exist
                let file = File::create(f).expect("unable to create file!");
                let mut w = BufWriter::new(file);
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
    fn try_run<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "{}", FileInfo::header(","))?;
        let dir = &self.path.clone().unwrap_or(".".into());
        self.read_dir(w, dir)?;
        w.flush().unwrap();
        Ok(())
    }

    /// Recursively reads the directory. Files, then directories
    fn read_dir<T: Write>(&self, w: &mut T, dir: &PathBuf) -> io::Result<()> {
        let (entries, failed): (Vec<_>, Vec<_>) = fs::read_dir(dir)?.partition(|e| e.is_ok());
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
        for f in files.into_iter() {
            let fil = FileInfo::from_entry(f);
            fil.unwrap().write_csvline(w, &self.path).unwrap();
        }
        // For directories, go deeper
        for other in &others {
            let ft = other.file_type()?;
            if ft.is_dir() {
                // go deeper
                self.read_dir(w, &other.path())?;
            } else {
                // Neither file nor directory. symlinks?
                eprintln!("WARN:::: {:?}", other.file_type());
                eprintln!("    :::: {:?}", other.metadata());
                eprintln!("    :::: {:?}", other);
            }
        }
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");
    cli.try_main().unwrap();
}
