#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg};
use env_logger::Env;
use nb::message::Message;
use nb::{Node, Result};
use serde_json::Deserializer;
use std::io::{stdin, stdout, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

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

    run_node(addr.to_owned());
}

fn run_node(addr: String) {
    let node = Arc::new(Mutex::new(Node::new()));

    let listener_node = node.clone();
    thread::spawn(move || handle_incoming_connections(listener_node, addr.clone()));

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
        const SEE_BLOCKCHAIN: &str = "list_blocks";
        const ADD_PEER: &str = "add_peer";
        const RESOLVE_CONFLICTS: &str = "resolve";
        const EXIT: &str = "exit";
        const HELP: &str = "help";
        const MINE: &str = "mine";

        {
            let mut node = node.lock().unwrap();
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
                ADD_PEER => {
                    if args.len() < 2 {
                        eprintln!("not enough arguments!");
                        continue;
                    }
                    let peer = *args.get(1).unwrap();
                    if false == node.add_peer(peer) {
                        eprintln!("fail to add peer");
                    }
                }
                RESOLVE_CONFLICTS => {
                    if node.resolve_conflicts() {
                        println!("node updated");
                    } else {
                        println!("node stays unchanged")
                    }
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
}

fn list_commands() {
    println!(concat!("blockchain node commands:\n",
    "  mine - mines a new block\n",
    "  new_trans [sender] [receiver] [amount] - adds a new transaction into the local blockchain\n",
    "  list_blocks - list the local chain blocks\n",
    "  add_peer [addr:port] - add one node as a peer\n",
    "  resolve - apply the consensus algorithm to resolve conflicts\n",
    "  exit - quit the program"));
}

fn handle_incoming_connections(node: Arc<Mutex<Node>>, addr: String) -> Result<()> {
    let listener = TcpListener::bind(&addr).expect("Fail to bind listener");
    for stream in listener.incoming() {
        debug!("new incoming connection");
        match stream {
            Ok(mut stream) => {
                debug!("waiting for request");
                // There should be only one request, but we have to deserialize from a stream in this way
                for request in Deserializer::from_reader(stream.try_clone()?).into_iter::<Message>()
                {
                    let request = request
                        .map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
                    debug!("request received {:?}", request);
                    if let Message::Request = request {
                        let response = Message::Response(node.lock().unwrap().get_blocks());
                        serde_json::to_writer(&mut stream, &response)?;
                        stream.flush()?;
                        debug!("response sent {:?}", response);
                    } else {
                        error!("invalid request");
                    }
                }
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
    Ok(())
}
