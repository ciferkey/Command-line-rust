use std::cmp::Ordering;
use std::fs;
use clap::{Parser, ArgAction};
use anyhow::{anyhow, Result, bail};
use std::io::{self, BufRead, BufReader, stdout};
use std::fs::{File};
use crate::Column::*;

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg()]
    file1: String,
    #[arg()]
    file2: String,
    #[arg(short('1'), action(ArgAction::SetFalse))]
    show_col1: bool,
    #[arg(short('2'), action(ArgAction::SetFalse))]
    show_col2: bool,
    #[arg(short('3'), action(ArgAction::SetFalse))]
    show_col3: bool,
    #[arg(short)]
    insensitive: bool,
    #[arg(short, long("output-delimiter"), default_value = "\t")]
    delimiter: String,
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| anyhow!("{filename}: {e}"))?,
        ))),
    }
}

fn run(args: Args) -> anyhow::Result<()> {
    let file1 = &args.file1;
    let file2 = &args.file2;

    if file1 == "-" && file2 == "-" {
        bail!(r#"Both input files cannot be STDIN ("-")"#);
    }

    let case = |line: String| {
        if args.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let mut lines1 = open(file1)?.lines().map_while(Result::ok).map(case);
    let mut lines2 = open(file2)?.lines().map_while(Result::ok).map(case);

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if args.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if args.show_col2 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if args.show_col3 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    if args.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        };

        if !columns.is_empty() {
            println!("{}", columns.join(&args.delimiter));
        }
    };

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(left), Some(right)) => {
                match left.cmp(right) {
                    Ordering::Less => {
                        print(Col1(left));
                        line1 = lines1.next();
                    }
                    Ordering::Equal => {
                        print(Col3(left));

                        line1 = lines1.next();
                        line2 = lines2.next();
                    }
                    Ordering::Greater => {
                        print(Col2(right));
                        line2 = lines2.next();
                    }
                }
            }
            (Some(left), None) => {
                print(Col1(left));
                line1 = lines1.next();
            }
            (None, Some(right)) => {
                print(Col2(right));
                line2 = lines2.next();
            }
            _ => (),
        };
    }

    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
