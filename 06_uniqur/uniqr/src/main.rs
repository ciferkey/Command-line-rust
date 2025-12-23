use clap::{arg, Parser};
use anyhow::{anyhow, Result};
use std::fs::{File};
use std::io::{self, BufRead, BufReader, Write};
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {

    #[arg(value_name = "IN_FILE", default_value = "-")]
    in_file: String,

    #[arg(value_name = "OUT_FILE",)]
    out_file: Option<String>,

    #[arg(short, long)]
    count: bool,
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(args: Args) -> Result<()> {

    let mut file = open(&args.in_file)
        .map_err(|e| anyhow!("{}: {e}", args.in_file))?;

    let mut out_file: Box<dyn Write> = match &args.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };

    let mut print = |num: u64, text: &str| -> Result<()> {
        if num > 0 {
            if args.count {
                write!(out_file, "{num:>4} {text}")?;
            } else {
                write!{out_file, "{text}"}?;
            }
        }
        Ok(())
    };

    let mut previous_line = String::new();
    let mut line = String::new();
    let mut counter = 0;

    loop {
        let bytes = file.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        if line.trim_end() != previous_line.trim_end() {
            print(counter, &previous_line)?;

            previous_line = line.clone();
            counter = 0;
        }

        counter += 1;
        line.clear();
    }

    print(counter, &previous_line)?;

    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
