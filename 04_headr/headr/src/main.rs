use anyhow::Result;
use clap::Parser;
use std::fs::{read, File};
use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(
        short('n'),
        long,
        default_value_t = 10,
        value_name = "LINES",
        value_parser = clap::value_parser!(u64).range(1..)

    )]
    lines: u64,

    #[arg(
        short('c'),
        long,
        value_name = "BYTES",
        conflicts_with("lines"),
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    bytes: Option<u64>,
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(args: Args) -> Result<()> {
    let num_files = args.files.len();

    for (file_num, filename) in args.files.iter().enumerate() {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {filename}: {err}"),
            Ok(mut reader) =>

                if num_files > 1 {
                    if file_num > 0 {
                        println!("");
                    }
                    println!("==> {filename} <==");
                }

                if let Some(num_bytes) = args.bytes {
                    let mut buffer = vec![0; num_bytes as usize];
                    let bytes_read = reader.read(&mut buffer)?;
                    print!(
                        "{}",
                        String::from_utf8_lossy(&buffer[..bytes_read])
                    );
                } else {
                    let mut line = String::new();

                    for _ in 0..args.lines {
                        let bytes = reader.read_line(&mut line)?;

                        if bytes == 0 {
                            break;
                        }

                        print!("{line}");
                        line.clear();
                    }
                }
            },
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
