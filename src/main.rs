#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate clap;
extern crate chrono;
extern crate csv;

use clap::App;
use std::io::{self, BufWriter, Write};
use std::process;
use std::fs::File;
use chrono::prelude::*;

const BUFSIZE: usize = 8192;

type CleanedRecord = Vec<String>;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let input_file = matches.value_of("input").unwrap_or("-");
    let cascade_p = matches.is_present("cascade");

    let date_options = [
        "%d/%m/%y",
        "%d %B %Y",
        "%d-%B-%y"
    ];

    let date_col: Option<Vec<usize>> = match matches.value_of("datecol") {
        Some(s) => { Some(parse_datecol(s)) }
        None => { None }
    };

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
                if let Some(pupil_num) = record.clone().get(9) {
                    if ! pupil_num.is_empty() {
                        process_line(record, &mut last_line, cascade_p, &date_col, &date_options);
                        writeln!(writer, "{}", json!(last_line)).unwrap();
                    }
                }
            }
            Err(err) => {
                eprintln!("error reading CSV from <stdin>: {}", err);
                process::exit(1);
            }
        }
    }
}

fn process_line(
    line: csv::StringRecord,
    last_line: &mut CleanedRecord,
    cascade_p: bool,
    date_col: &Option<Vec<usize>>,
    date_options: &[&str])
{
    for i in 0..(line.len()) {
        if let Some(field) = line.get(i) {
            if field.len() > 0 || ! cascade_p {
                last_line[i].clear();

                if let Some(col) = date_col {
                    if col.contains(&i) {
                        if field.is_empty() {
                            last_line[i].push_str("");
                            continue;
                        }
                        let mut iso_date = None;
                        for format in date_options.iter() {
                            match NaiveDate::parse_from_str(field, format) {
                                Ok(date) => {
                                    iso_date = Some(date);
                                    break;
                                }
                                Err(_) => {}
                            }
                        }

                        match iso_date {
                            Some(date) => {
                                last_line[i].push_str(&date.format("%Y-%m-%d")
                                                           .to_string());
                                continue;
                            }
                            None => {
                                /*
                                eprintln!("error converting date");
                                process::exit(1);
                                */
                                last_line[i].push_str("");
                                continue;
                            }
                        }
                    }
                }

                last_line[i].push_str(field);
            }
        }
    }
}

fn parse_datecol(raw: &str) -> Vec<usize> {
    raw.split(",").map(|c|
        match c.parse::<usize>() {
            Ok(i) => {
                if i < 1 {
                    eprintln!("Expected a positive integer for --datecol: numbering starts at `1`.");
                    process::exit(1);
                }
                else {
                    i - 1
                }
            }
            Err(_) => {
                eprintln!("Couldn't not parse column number `{}` for --datecol: expected positive integer.", c);
                process::exit(1);
            }
        }
    ).collect()
}
