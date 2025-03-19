use std::fs;
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
        for f in fs::read_dir(dir)? {
            let entry = f?;
            let ft = entry.file_type()?;
            if ft.is_file() {
                let fil = FileInfo::from_entry(entry);
                fil.unwrap().write_csvline(w).unwrap();
            } else if ft.is_dir() {
                // go deeper
                self.read_dir(w, &entry.path())?;
            } else {
                eprintln!("WARN:::: {:?}", entry.file_type());
                eprintln!("WARN:::: {:?}", entry.metadata());
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
