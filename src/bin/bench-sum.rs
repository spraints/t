// version of parse and sum:
//  puts T::Data.new(ENV["T_DATA_FILE"]).entries.inject(0) { |sum, e| sum + e.minutes }

use t::parser::parse_entries;

fn main() {
    let data_file = std::env::var("T_DATA_FILE").unwrap();
    let f = std::fs::File::open(data_file).unwrap();
    let entries = parse_entries(f).unwrap();
    let sum = entries
        .into_iter()
        .fold(0, |sum, entry| sum + entry.minutes());
    println!("{}", sum);
}
