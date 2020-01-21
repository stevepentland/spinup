use clap::{App, Arg};
use libspinup::generate_configurations;

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let matches = App::new("Generate")
        .about("Util to generate from one config to the others")
        .arg(Arg::with_name("source").required(true).index(1))
        .get_matches();

    let source_format = matches.value_of("source").unwrap();
    generate_configurations(source_format);
}
