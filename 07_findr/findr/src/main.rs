use std::any::Any;
use regex::Regex;
use clap::{arg, Parser, builder::PossibleValue, ValueEnum, ArgAction};
use anyhow::Result;
use std::fs::{File};
use std::io::{self, BufRead, BufReader, Write};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Eq, PartialEq, Clone)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[EntryType::Dir, EntryType::File, EntryType::Link]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            EntryType::Dir => PossibleValue::new("d"),
            EntryType::File => PossibleValue::new("f"),
            EntryType::Link => PossibleValue::new("l"),
        })
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {

    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    #[arg(
        short,
        long("name"),
        value_name = "NAME",
        value_parser(Regex::new),
        action(ArgAction::Append),
        num_args(0..),
    )]
    names: Vec<Regex>,

    #[arg(
        short('t'),
        long("type"),
        value_name = "TYPE",
        value_parser(clap::value_parser!(EntryType)),
        action(ArgAction::Append),
        num_args(0..),
    )]
    entry_types: Vec<EntryType>,
}

fn print(args: &Args, entry: DirEntry) -> Result<()>  {

    if !args.names.is_empty() {
        let path = entry.file_name().to_string_lossy();

        let is_match = args.names.iter().map(|regex| regex.is_match(&path)).any(|v| v);
        if !is_match {
            return Ok(())
        }
    }

    if !args.entry_types.is_empty() {
        let file_type = entry.file_type();

        let matches = args.entry_types.iter().map(|entry_type| {
            match entry_type {
                EntryType::Dir => file_type.is_dir(),
                EntryType::File => file_type.is_file(),
                EntryType::Link => file_type.is_symlink(),
            }
        }).any(|v| v);
        
        if !matches {
            return Ok(())
        }
    }

    println!("{}", entry.path().display());

    Ok(())
}

fn run(args: Args) -> Result<()> {
    for path in &args.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{e}"),
                Ok(entry) => print(&args, entry)?,
            }
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