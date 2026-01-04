use anyhow::{Result, bail};
use clap::Parser;
use std::fs::{File};
use std::io::{self, BufRead, BufReader, stdout};
use std::ops::Range;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug, clap::Args)]
#[group[required = true, multiple = false]]
struct ArgsExtract {

    #[arg(short, long, value_name = "FIELDS",)]
    fields: Option<String>,

    #[arg(short, long, value_name = "BYTES",)]
    bytes: Option<String>,

    #[arg(short, long, value_name = "CHARS",)]
    chars: Option<String>,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(
        short,
        long,
        default_value = "\t",
        value_name = "DELIMITER",
    )]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    char_pos.iter()
        .map(|range| line.chars()
            .skip(range.start)
            .take(range.len())
            .collect::<String>())
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    byte_pos.iter()
        .map(|range| {
            let bytes: Vec<u8> = line.bytes()
                .skip(range.start)
                .take(range.len())
                .collect();

            String::from_utf8_lossy(&bytes).into_owned()
        })
        .collect()
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]
) -> Vec<String> {
    let fields: Vec<_> = record.iter().collect();
    field_pos.iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| fields.get(i)))
        .map(|s| (*s).to_string())
        .collect()
}

fn check_list_value(whole: &str, part: &str) -> Result<usize> {
    if part.starts_with("+") {
        bail!(r#"illegal list value: "{whole}""#);
    }

    let location: usize = match part.parse() {
        Ok(l) => l,
        Err(e) => bail!(r#"illegal list value: "{whole}""#)
    };

    if location == 0 {
        bail!(r#"illegal list value: "{part}""#);
    }

    Ok(location)
}

fn parse_pos_part(part: &str) -> Result<Range<usize>>{

    let parts = part.split("-").collect::<Vec<&str>>();

    match parts.as_slice() {
        [] => {
            bail!(r#"range has no parts"#);
        },
        [first] => {

            let location = check_list_value(part, first)?;

            Ok(location-1..location)
        },
        [first, second] => {
            let start = check_list_value(part, first)?;
            let end: usize = check_list_value(part, second)?;

            if start >= end {
                bail!(r#"First number in range ({start}) must be lower than second number ({end})"#);
            }

            Ok(start-1..end)
        },
        [_, _, ..] => {
            bail!(r#"range has too many parts {}"#, part);
        },
    }

}

fn parse_pos(range: String) -> Result<PositionList> {
    if range == "" {
        bail!(r#"must provide a range"#);
    }

    range.split(",").map(|part| parse_pos_part(part)).collect()
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn run(args: Args) -> Result<()> {

    if args.delimiter.as_bytes().len() != 1 {
        bail!(r#"--delim "{}" must be a single byte"#, args.delimiter);
    }

    let delimiter: u8 = *args.delimiter.as_bytes().first().unwrap();

    let extract = if let Some(fields) =
        args.extract.fields.map(parse_pos).transpose()?
    {
        Extract::Fields(fields)
    } else if let Some(bytes) =
        args.extract.bytes.map(parse_pos).transpose()?
    {
        Extract::Bytes(bytes)
    } else if let Some(chars) =
        args.extract.chars.map(parse_pos).transpose()?
    {
        Extract::Chars(chars)
    } else {
        unreachable!("Must have --fields, --bytes, or --chars");
    };

    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => match &extract {
                Extract::Fields(field_pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(delimiter)
                        .has_headers(false)
                        .from_reader(file);

                    let mut writer = WriterBuilder::new()
                        .delimiter(delimiter)
                        .from_writer(io::stdout());

                    for record in reader.records() {
                        let fields = extract_fields(&record?, field_pos);
                        writer.write_record(fields)?;
                    }
                }
                Extract::Bytes(byte_pos) => {
                    for line in file.lines() {
                        let line = &line?;
                        println!("{}", extract_bytes(line, byte_pos));
                    }
                }
                Extract::Chars(char_pos) => {
                    for line in file.lines() {
                        let line = &line?;
                        println!("{}", extract_chars(line, char_pos));
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

#[cfg(test)]
mod unit_tests {
    use super::{parse_pos, extract_chars, extract_bytes, extract_fields};
    use csv::StringRecord;

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(
            extract_fields(&rec, &[0..1, 2..3]),
            &["Captain", "12345"]
        );
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(
            extract_chars("ábc", &[0..1, 1..2, 4..5]),
            "áb".to_string()
        );
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "0""#
        );

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "0""#
        );

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1""#,
        );

        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a""#
        );

        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a""#
        );

        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-a""#,
        );

        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a-1""#,
        );

        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());

        let res = parse_pos(",".to_string());
        assert!(res.is_err());

        let res = parse_pos("1,".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}