use clap::Parser;
use anyhow::Result;

use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(
        short('n'),
        long("number"),
        conflicts_with("number_nonblank_lines"),
    )]
    number_lines: bool,

    #[arg(short('b'), long("number-nonblank"))]
    number_nonblank_lines: bool,
}

// #[derive(Debug)]
// struct Args {
//     files: Vec<String>,
//     number_lines: bool,
//     number_nonblank_lines: bool,
// }
//
// fn get_args() -> Args {
//     let matches = Command::new("catr")
//         .version("0.1.0")
//         .author("Ken Youens-Clark <kyclark@gmail.com>")
//         .about("Rust version of `cat`")
//         .arg(
//             Arg::new("files")
//                 .value_name("FILES")
//                 .help("")
//                 .num_args(1..)
//         )
//         .arg(
//             Arg::new
//         )
//         .get_matches();
//
//     Args {
//         files: matches.get_many("files").parse().unwrap(),
//         number_lines: matches.get_flag("number_lines"),
//         number_nonblank_lines: matches.get_flag("number_nonblank_lines")
//     }
// }

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(args: Args) -> Result<()> {
    for filename in args.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {filename}: {err}"),
            Ok(reader) => {
                let mut line_number = 1;
                for line in reader.lines() {
                    let line = line?;

                    if args.number_lines {
                        println!("{:>6}\t{line}", line_number);
                    } else if args.number_nonblank_lines {
                        if line == "" {
                            println!();
                            line_number -=1; // Offset the increment we will do later.
                        } else {
                            println!("{line_number:>6}\t{line}");
                        }
                    } else {
                        println!("{line}");
                    }

                    line_number += 1;
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
