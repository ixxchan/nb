#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg};
use env_logger::Env;
use nb::message::{Request, Response};
use nb::{Node, Result};
use serde_json::Deserializer;
use std::io::{stdin, stdout, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    let node = Arc::new(Mutex::new(Node::new(addr.clone())));

    let listener_node = node.clone();
    thread::spawn(move ||
        {
            let node = listener_node;
            let addr = addr.clone();
            loop {
                let _result = handle_incoming_connections(node.clone(), addr.clone());
            }
        });
    let broadcast_node = node.clone();
    thread::spawn(move || handle_broadcast(broadcast_node));
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
                    node.create_and_add_new_transaction(sender, receiver, amount);
                }
                MINE => {
                    node.mine();
                    debug!("Mined!!!")
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
                    if false == node.detect_peer(peer) {
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
                // There should be only one request, but we have to deserialize from a stream in this way
                for request in Deserializer::from_reader(stream.try_clone()?).into_iter::<Request>()
                {
                    let request = request
                        .map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
                    debug!("request received {:?}", request);
                    // try to add a new peer from every request
                    let mut node = node.lock().unwrap();
                    let peer_info = request.get_peer_info();
                    if node.add_peer(peer_info.clone()) {
                        info!("Add one new peer: {:?}", peer_info);
                    }
                    let my_info = node.get_basic_info();
                    let response = match request {
                        Request::Hello(peer_info) => {
                            info!("Get Hello from {:?}, simply ack it", peer_info);
                            Response::Ack(my_info)
                        }
                        Request::NewTransaction(peer_info, transaction) => {
                            info!(
                                "Get NewTransaction from {:?}, add the transaction and ack it",
                                peer_info
                            );
                            node.handle_incoming_transaction(transaction);
                            Response::Ack(my_info)
                        }
                        Request::NewBlock(peer_info, new_block) => {
                            info!(
                                "Get NewBlock from {:?}, validate it and possibly add it to our chain",
                                peer_info
                            );
                            node.handle_incoming_block(new_block);
                            Response::Ack(my_info)
                        }
                        Request::HowAreYou(peer_info) => {
                            info!(
                                "Get HowAreYou from {:?}, will respond with all my blocks",
                                peer_info
                            );
                            Response::MyBlocks(node.get_basic_info(), node.get_blocks())
                        }
                    };
                    serde_json::to_writer(&mut stream, &response)?;
                    stream.flush()?;
                    debug!("response sent {:?}", response);
                    break;
                }
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_broadcast(node: Arc<Mutex<Node>>) {
    loop {
        node.lock().unwrap().try_fetch_one_broadcast();
        thread::sleep(Duration::from_secs(3));
    }
}
