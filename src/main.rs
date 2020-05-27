#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg};
use env_logger::Env;
use nb::Node;
use std::io::{stdin, stdout, Write};

fn main() {
    let matches = App::new("nb")
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

    env_logger::from_env(Env::default().default_filter_or("debug")).init();

    info!("nb {}", env!("CARGO_PKG_VERSION"));
    info!("Listening on {}", addr);

    run_node(addr);
}

fn run_node(addr: &str) {
    let mut node = Node::new();

    loop {
        let mut input = String::new();
        // a prompt for input
        print!("> ");
        stdout().flush().expect("flush error");

        stdin().read_line(&mut input).expect("cannot read input");

        let input = input.trim();
        let args: Vec<&str> = input.split_whitespace().collect();
        let command = match args.get(0) {
            Some(value) => *value,
            None => {
                continue;
            }
        };
        debug!("args: {:?}, command: {}", args, command);

        const NEW_TRANS: &str = "new_trans";
        const ADD_BLOCK: &str = "add_block";
        const SEE_BLOCKCHAIN: &str = "list_blocks";
        const ADD_PEER: &str = "add_peer";
        const LIST_PEERS: &str = "list_peers";
        const EXIT: &str = "exit";
        const HELP: &str = "help";
        const MINE: &str = "mine";

        match command {
            NEW_TRANS => {
                if args.len() < 4 {
                    eprintln!("not enough arguments!");
                    continue;
                }
                let sender = *args.get(1).unwrap();
                let receiver = *args.get(2).unwrap();
                let amount: i64;
                match (*args.get(3).unwrap()).parse() {
                    Ok(num) => amount = num,
                    Err(_) => {
                        eprintln!("illegal amount!");
                        continue;
                    }
                };
                node.new_transaction(sender, receiver, amount);
            }
            MINE => {
                node.mine();
            }
            SEE_BLOCKCHAIN => {
                node.display();
            }
            HELP => {
                list_commands();
            }
            EXIT => {
                break;
            }
            _ => {
                println!("Command not found. Type 'help' to list commands.");
            }
        }
    }
}

pub fn list_commands() {
    println!(concat!("blockchain node commands:\n",
    "  mine - mines a new block\n",
    "  new_trans [sender] [receiver] [amount] - adds a new transaction into the local blockchain\n",
    "  list_blocks - list the local chain blocks\n",
    "  exit - quit the program"));
    //    println!("add_peer - add one node as a peer");
    //    println!("Example: add_peer 172.17.0.10\n");
    //    println!("list_peers - list the peers\n");
}
