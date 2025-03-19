use std::fs;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;

use kagirfilelists::FileInfo;

fn do_main(dir: PathBuf) -> io::Result<()> {
    let mut w = BufWriter::new(io::stdout());
    for f in fs::read_dir(dir)? {
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

fn main() {
    let args = Cli::parse();
    println!("{args:?}");
    do_main(args.path.unwrap_or(".".into())).unwrap();
}
