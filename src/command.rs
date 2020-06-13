use crate::*;
use colored::Colorize;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::Sender;

pub enum Command {
    NewTrans(String, String, i64), // sender, receiver, amount
    Display,
    AddPeer(String),
    DisplayPeers,
    Resolve,
    Mine,
}

const NEW_TRANS: &str = "new_trans";
const SEE_BLOCKCHAIN: &str = "list_blocks";
const ADD_PEER: &str = "add_peer";
const LIST_PEERS: &str = "list_peers";
const RESOLVE_CONFLICTS: &str = "resolve";
const EXIT: &str = "exit";
const HELP: &str = "help";
const MINE: &str = "mine";

pub fn handle_input_commands(sender: Sender<Event>) {
    loop {
        let mut input = String::new();
        // a prompt for input
        print!("{}", "> ".color(PROMPT_COLOR).bold());
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
        let mut event_cmd = None;
        match command {
            NEW_TRANS => {
                if args.len() < 4 {
                    eprintln!("{}", "not enough arguments!".color(ERR_COLOR));
                    continue;
                }
                let sender = *args.get(1).unwrap();
                let receiver = *args.get(2).unwrap();
                let amount: i64;
                match (*args.get(3).unwrap()).parse() {
                    Ok(num) => amount = num,
                    Err(_) => {
                        eprintln!("{}", "illegal amount!".color(ERR_COLOR));
                        continue;
                    }
                };
                event_cmd = Some(Command::NewTrans(
                    sender.to_owned(),
                    receiver.to_owned(),
                    amount,
                ))
            }
            MINE => {
                event_cmd = Some(Command::Mine);
                debug!("{}", "Mined!!!".color(MSG_COLOR))
            }
            SEE_BLOCKCHAIN => {
                event_cmd = Some(Command::Display);
            }
            ADD_PEER => {
                if args.len() < 2 {
                    eprintln!("{}", "not enough arguments!".color(ERR_COLOR));
                    continue;
                }
                let peer = *args.get(1).unwrap();
                event_cmd = Some(Command::AddPeer(peer.to_owned()));
            }
            LIST_PEERS => {
                event_cmd = Some(Command::DisplayPeers);
            }
            RESOLVE_CONFLICTS => {
                event_cmd = Some(Command::Resolve);
            }
            HELP => {
                list_commands();
            }
            EXIT => {
                break;
            }
            _ => {
                eprintln!(
                    "{}",
                    "Command not found. Type 'help' to list commands.".color(ERR_COLOR)
                );
            }
        }
        if let Some(event_cmd) = event_cmd {
            sender.send(Event::Command(event_cmd)).unwrap();
        }
    }
}

fn list_commands() {
    println!(
        "{}",
        concat!("blockchain node commands:\n",
        "  mine - mines a new block\n",
        "  new_trans [sender] [receiver] [amount] - adds a new transaction into the local blockchain\n",
        "  list_blocks - list the local chain blocks\n",
        "  add_peer [addr:port] - add one node as a peer\n",
        "  list_peers - list the node's peers\n",
        "  resolve - apply the consensus algorithm to resolve conflicts\n",
        "  exit - quit the program")
            .color(MSG_COLOR)
    );
}
