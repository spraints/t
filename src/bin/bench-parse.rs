// version of parse only:
//  puts T::Data.new(ENV["T_DATA_FILE"]).entries.size

use t::parser::parse_entries;
use t::timesource::real_time::DefaultTimeSource;

fn main() {
    let data_file = std::env::var("T_DATA_FILE").unwrap();
    let f = std::fs::File::open(data_file).unwrap();
    let entries = parse_entries(f, &DefaultTimeSource).unwrap();
    println!("{}", entries.len());
}
