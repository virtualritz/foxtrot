use step::parse::parse_file_at_path;
use std::time::{SystemTime};

fn main() {
    let start = SystemTime::now();
    
    let filename = "/Users/Henry Heffan/Desktop/foxtrot/Kondo_only_data.step";
    parse_file_at_path(filename);

    let end = SystemTime::now();
    let since_the_epoch = end.duration_since(start).expect("Time went backwards");
    println!("time {:?}", since_the_epoch);
}
