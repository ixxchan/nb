#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg};
use sb::Blockchain;
use env_logger::Env;

fn main() {
    let matches = App::new("sb")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A simple blockchain node")
        .setting(AppSettings::DisableHelpSubcommand)
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .value_name("IP-PORT")
                .default_value("127.0.0.1:4000")
                .help("the node's address"),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap();

    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!("sb {}", env!("CARGO_PKG_VERSION"));
    info!("Listening on {}", addr);

    run_node(addr);
}

fn run_node(addr: &str) {
    unimplemented!()
}
