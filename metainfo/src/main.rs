use std::env::args;

use bencode::FromBencode;
use rusty_tracker::{MetaInfoFiles, MetaInfoFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = args().nth(1).expect("Usage: <file_path>");

    let file = std::fs::read(arg)
        .expect("Could not read the file");

    let data = bencode::from_vec(file).unwrap();

    println!("{:?}", data);

    let meta_files: Result<MetaInfoFiles, _> = FromBencode::from_bencode(&data);

    if let Ok(res) = meta_files {
        println!("MetaInfo Mode  Many Files: {:?}", res);
        return Ok(());
    }

    let meta_file: Result<MetaInfoFile, _> = FromBencode::from_bencode(&data);
    
    if let Ok(res) = meta_file {
        println!("MetaInfo Mode Single File: {:?}", res);
    } else {
        println!("Invalid metainfo file");
    }

    Ok(())
}