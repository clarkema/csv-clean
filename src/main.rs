#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate clap;
extern crate csv;

use std::io::{self, BufWriter, Write};
use std::process;
use std::fs::File;

const BUFSIZE: usize = 8192;

type CleanedRecord = Vec<String>;

fn main() {
    let matches = clap_app!(csv_clean =>
        (version: "pre")
        (author: "Mike Clarke <mike@lambdafunctions.com>")
        (@arg input: "Input file.  Omit or use '-' for STDIN.")
    ).get_matches();

    let input_file = matches.value_of("input").unwrap_or("-");

    let mut last_line = CleanedRecord::new();

    let reader: Box<io::Read> = if input_file == "-" {
        Box::new(io::stdin())
    }
    else {
        Box::new(File::open(input_file).unwrap())
    };

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    if let Ok(headers) = rdr.headers() {
        for _ in headers.iter() {
            last_line.push(String::new());
        }
    }

    let mut writer = BufWriter::with_capacity(BUFSIZE, io::stdout());

    for result in rdr.records() {
        match result {
            Ok(record) => {
                process_line(record, &mut last_line);
                writeln!(writer, "{}", json!(last_line)).unwrap();
            }
            Err(err) => {
                eprintln!("error reading CSV from <stdin>: {}", err);
                process::exit(1);
            }
        }
    }
}

fn process_line(line: csv::StringRecord, last_line: &mut CleanedRecord) {
    for i in 0..(line.len()) {
        if let Some(field) = line.get(i) {
            if field.len() > 0 {
                last_line[i].clear();
                last_line[i].push_str(field);
            }
        }
    }
}
