use std::fs;
use std::io;
use std::io::{BufWriter, Write};

use kagirfilelists::FileInfo;

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

fn main() {
    do_main().unwrap();
}
