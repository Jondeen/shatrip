extern crate clap;

use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufRead, SeekFrom, Seek};
use sha2::{Sha256, Digest};
use clap::Parser;

fn write_hashes(input_filename: &str, hash_filename: &str, chunk_size: usize) -> io::Result<()> {
    let mut input_file = File::open(input_filename)?;
    let mut hash_file = File::create(hash_filename)?;
    let mut buffer = vec![0u8; chunk_size];

    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let mut hasher = Sha256::new();
        hasher.update(&buffer[0..bytes_read]);
        let result = hasher.finalize();
        
        writeln!(&mut hash_file, "{:x}", result)?;
    }

    Ok(())
}

fn check_hashes(input_filename: &str, hash_filename: &str, chunk_size: usize) -> io::Result<()> {
    let mut input_file = File::open(input_filename)?;
    let hash_file = File::open(hash_filename)?;
    let reader = BufReader::new(hash_file);
    let mut buffer = vec![0u8; chunk_size];

    for (line_num, line) in reader.lines().enumerate() {
        let expected_hash = line?;
        let bytes_read = input_file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        let mut hasher = Sha256::new();
        hasher.update(&buffer[0..bytes_read]);
        let result = hasher.finalize();

        if format!("{:x}", result) != expected_hash {
            println!("Mismatch at chunk {}", line_num);
        }
    }

    Ok(())
}


fn extract_chunk(input_filename: &str, output_chunk_filename: &str, chunk_number: usize, chunk_size: usize) -> io::Result<()> {
    let mut input_file = File::open(input_filename)?;
    let mut output_chunk_file = File::create(output_chunk_filename)?;

    let start_pos = chunk_number * chunk_size;
    input_file.seek(SeekFrom::Start(start_pos as u64))?;
    let mut buffer = vec![0; chunk_size];
    input_file.read_exact(&mut buffer)?;
    output_chunk_file.write_all(&buffer)?;

    Ok(())
}

fn fix_chunk(input_filename: &str, source_chunk_filename: &str, chunk_number: usize, chunk_size: usize) -> io::Result<()> {
    let mut input_file = File::open(input_filename)?;
    let mut source_chunk_file = File::open(source_chunk_filename)?;

    let start_pos = chunk_number * chunk_size;
    input_file.seek(SeekFrom::Start(start_pos as u64))?;
    let mut buffer = vec![0; chunk_size];
    source_chunk_file.read_exact(&mut buffer)?;
    input_file.write_all(&buffer)?;

    Ok(())
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Your Name", about = "File Hash Checker")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Write(WriteOpts),
    Check(CheckOpts),
    Extract(ExtractOpts),
    Fix(FixOpts),
}

#[derive(Parser)]
struct WriteOpts {
    input: String,
    hashes: String,
    #[clap(default_value = "1024")]
    chunk_size: usize,
}

#[derive(Parser)]
struct CheckOpts {
    input: String,
    hashes: String,
    #[clap(default_value = "1024")]
    chunk_size: usize,
}

#[derive(Parser)]
struct ExtractOpts {
    input: String,
    chunk_file: String,
    chunk_number: usize,
    #[clap(default_value = "1024")]
    chunk_size: usize,
}

#[derive(Parser)]
struct FixOpts {
    input: String,
    chunk_file: String,
    chunk_number: usize,
    #[clap(default_value = "1024")]
    chunk_size: usize,
}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Write(w) => write_hashes(&w.input, &w.hashes, w.chunk_size)?,
        SubCommand::Check(c) => check_hashes(&c.input, &c.hashes, c.chunk_size)?,
        SubCommand::Extract(e) => extract_chunk(&e.input, &e.chunk_file, e.chunk_number, e.chunk_size)?,
        SubCommand::Fix(f) => fix_chunk(&f.input, &f.chunk_file, f.chunk_number, f.chunk_size)?,
    }

    Ok(())
}
