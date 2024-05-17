use std::fs::File;
use std::io::Read;
use std::time::SystemTime;

use clap::{App, Arg};
use express::parse::{parse, strip_comments_and_lower};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("parse_exp")
        .author("Matt Keeter <matt@formlabs.com>")
        .about("Parses an EXPRESS file")
        .arg(Arg::with_name("input").takes_value(true).required(true))
        .arg(
            Arg::with_name("quiet")
                .short('q')
                .long("quiet")
                .help("disable output"),
        )
        .arg(Arg::with_name("output").takes_value(true))
        .get_matches();
    let input = matches.value_of("input").expect("Could not get input file");

    let mut f = File::open(input).expect("file opens");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("read ok");

    let start = SystemTime::now();
    let s = strip_comments_and_lower(&buffer);
    let mut parsed = parse(&s);

    let end = SystemTime::now();
    let since_the_epoch = end.duration_since(start).expect("Time went backwards");
    eprintln!("time {:?}", since_the_epoch);

    match parsed {
        Err(e) => eprintln!("Got err {:?}", e),
        Ok((_, ref mut p)) => match matches.value_of("output") {
            Some(o) => std::fs::write(o, format!("Parse tree:\n{:#?}", p))?,
            _ => {
                if !matches.is_present("quiet") {
                    println!("Parse tree:\n{:#?}", parsed);
                }
            }
        },
    };
    Ok(())
}
