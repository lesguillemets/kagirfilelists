use std::fs;
use std::fs::DirEntry;
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
    fn try_run(&self) -> io::Result<()> {
        let mut w = BufWriter::new(io::stdout());
        writeln!(&mut w, "{}", FileInfo::header(","))?;
        let dir = &self.path.clone().unwrap_or(".".into());
        self.read_dir(&mut w, dir)?;
        w.flush().unwrap();
        Ok(())
    }

    fn read_dir<T: Write>(&self, w: &mut T, dir: &PathBuf) -> io::Result<()> {
        let (entries, failed): (Vec<_>, Vec<_>) = fs::read_dir(dir)?.partition(|e| e.is_ok());
        for fail in &failed {
            eprintln!("ERROR: {fail:?}");
        }
        let (files, others): (Vec<DirEntry>, Vec<DirEntry>) =
            entries.into_iter().map(|e| e.unwrap()).partition(|e| {
                e.file_type()
                    .unwrap_or_else(|_| panic!("Cli::read_dir,file_type on {e:?}"))
                    .is_file()
            });
        for f in files.into_iter() {
            let fil = FileInfo::from_entry(f);
            fil.unwrap().write_csvline(w).unwrap();
        }
        for other in &others {
            let ft = other.file_type()?;
            if ft.is_dir() {
                // go deeper
                self.read_dir(w, &other.path())?;
            } else {
                eprintln!("WARN:::: {:?}", other.file_type());
                eprintln!("WARN:::: {:?}", other.metadata());
            }
        }
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");
    cli.try_run().unwrap();
}
