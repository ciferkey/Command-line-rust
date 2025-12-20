use anyhow::Result;
use clap::Parser;
use std::fs::{File};
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long, default_value_t = false)]
    lines: bool,

    #[arg(short, long, default_value_t = false)]
    words: bool,

    #[arg(short('c'), long, default_value_t = false)]
    bytes: bool,

    #[arg(short('m'), long, default_value_t = false, conflicts_with("bytes"))]
    chars: bool,
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize
}

fn count(mut file: impl BufRead) -> Result<FileInfo> {

    let mut buffer = String::new();
    let _ = file.read_to_string(&mut buffer);

    let mut num_lines = buffer.lines().count();
    let mut num_words = buffer.split_whitespace().count();
    let mut num_bytes = buffer.bytes().count();
    let mut num_chars = buffer.chars().count();

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(mut args: Args) -> Result<()> {
    if [args.words, args.bytes, args.chars, args.lines]
    .iter()
    .all(|v| v == &false) {
        args.lines = true;
        args.words = true;
        args.bytes = true;
    }

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("Failed to open {filename}: {err}"),
            Ok(mut reader) => {

                let file_info = count(reader)?;

                if args.lines {
                    print!("{:>8}", file_info.num_lines);
                }

                if args.words {
                    print!("{:>8}", file_info.num_words);
                }

                if args.bytes {
                    print!("{:>8}", file_info.num_bytes);
                } else if args.chars {
                    print!("{:>8}", file_info.num_chars);
                }

                if filename != "-" {
                    println!(" {}", filename);
                }

                total_lines += file_info.num_lines;
                total_words += file_info.num_words;
                total_bytes += file_info.num_bytes;
                total_chars += file_info.num_chars;
            }
        }
    }

    if args.files.len() > 1 {
        if args.lines {
            print!("{:>8}", total_lines);
        }

        if args.words {
            print!("{:>8}", total_words);
        }

        if args.bytes {
            print!("{:>8}", total_bytes);
        } else if args.chars {
            print!("{:>8}", total_chars);
        }

        println!(" total");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world.\nI just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}