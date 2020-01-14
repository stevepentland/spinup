#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use libspinup::{run_app, RunConfig};

const DEFAULT_LOG_LEVEL: &str = "warn";

#[cfg_attr(tarpaulin, skip)]
#[tokio::main]
async fn main() {
    let mut app = App::new("Spinup")
        .version(crate_version!())
        .author("Steve Pentland")
        .about("Helps you spin up your new environment")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Increase the verbosity of the program. This may be specified multiple times")
                .multiple(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppress all program output")
                .multiple(false)
                .takes_value(false)
                .conflicts_with("verbose"),
        )
        .arg(
            Arg::with_name("no-packages")
                .short("P")
                .long("no-packages")
                .help("Don't install packages")
                .multiple(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("no-files")
                .short("F")
                .long("no-files")
                .help("Don't download files")
                .multiple(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("no-snaps")
                .short("S")
                .long("no-snaps")
                .help("Don't install snap packages")
                .multiple(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("print-parsed")
                .long("print-parsed")
                .help("Print the parsed config")
                .multiple(false)
                .takes_value(false)
                .hidden(true),
        )
        .arg(
            Arg::with_name("no-commands")
                .short("C")
                .long("no-commands")
                .help("Don't run custom commands or command sets")
                .multiple(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("CONFIG")
                .help("The input configuration file")
                .required(true)
                .index(1),
        );

    if cfg!(debug_assertions) {
        // TODO: Keep only for development
        app = app.arg(
            Arg::with_name("generate")
                .long("generate")
                .short("g")
                .takes_value(false)
                .hidden(true),
        );
    }

    let matches = app.get_matches();
    let run_config = build_run_config(matches);

    let res = run_app(run_config).await;

    ::std::process::exit(match res {
        Ok(()) => 0,
        Err(e) => {
            error!("{}", e);
            1
        }
    });
}

fn build_run_config(matches: clap::ArgMatches) -> RunConfig {
    let log_level = get_log_level(
        matches.occurrences_of("verbose"),
        matches.is_present("quiet"),
    );
    RunConfig::new(
        matches.value_of("CONFIG").unwrap().to_owned(),
        log_level,
        !matches.is_present("no-packages"),
        !matches.is_present("no-files"),
        !matches.is_present("no-snaps"),
        !matches.is_present("no-commands"),
        matches.is_present("print-parsed"),
    )
}

fn get_log_level(verbosity: u64, is_quiet: bool) -> &'static str {
    if is_quiet {
        return "off";
    }
    match verbosity {
        0 => DEFAULT_LOG_LEVEL,
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}
