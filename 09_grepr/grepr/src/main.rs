use std::fs;
use clap::Parser;
use regex::{RegexBuilder, Regex};
use anyhow::{anyhow, Result, bail};
use walkdir::{WalkDir, DirEntry};
use std::io::{self, BufRead, BufReader, stdout};
use std::fs::{File};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "PATTERN")]
    pattern: String,

    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long, value_name = "INSENSITIVE",)]
    insensitive: bool,

    #[arg(short, long, value_name = "RECURSIVE",)]
    recursive: bool,

    #[arg(short, long, value_name = "COUNT",)]
    count: bool,

    #[arg(short('v'), long("invert-match"), value_name = "INVERT",)]
    invert: bool,
}

fn find_files(paths: &[String], recursive: bool) -> Vec<Result<String>> {
    let mut results = vec![];

    for path in paths {
        match path.as_str() {
            "-" => results.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if recursive {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                            {
                                results.push(Ok(entry
                                    .path()
                                    .display()
                                    .to_string()));
                            }
                        } else {
                            results
                                .push(Err(anyhow!("{path} is a directory")));
                        }
                    } else if metadata.is_file() {
                        results.push(Ok(path.to_string()));
                    }
                }
                Err(e) => results.push(Err(anyhow!("{path}: {e}"))),
            },
        }
    }

    results
}

fn find_lines<T: BufRead>(mut file: T, pattern: &Regex, invert: bool,
) -> Result<Vec<String>> {

    let mut matches: Vec<String> = Vec::new();

    let mut line = String::new();

    loop {
        match file.read_line(&mut line) {
            Ok(size) => {

                if size == 0 {
                    break
                }

                if pattern.is_match(&line) ^ invert {
                    matches.push(line.clone())
                }
            },
            Err(e) => break
        }

        line.clear();
    }

    Ok(matches)
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(args: Args) -> anyhow::Result<()> {
    let pattern = RegexBuilder::new(&args.pattern)
        .case_insensitive(args.insensitive)
        .build()
        .map_err(|_| anyhow!(r#"Invalid pattern "{}""#, args.pattern))?;

    let entries = find_files(&args.files, args.recursive);
    for entry in &entries {
        match entry {
            Err(e) => eprintln!("{e}"),
            Ok(filename) => match open(&filename) {
                Ok(file) => {
                    let matches = find_lines(file, &pattern, args.invert)?;

                    let prefix = if args.files.len() > 1 || args.recursive {
                        filename.to_owned() + ":"
                    } else {
                        String::new()
                    };

                    if args.count {
                        let count = matches.len();
                        println!("{prefix}{count}");
                    } else {
                        for _match in matches {
                            print!("{prefix}{_match}")
                        }
                    }
                }
                Err(e) => eprintln!("{filename}: {e}")
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

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files =
            find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}