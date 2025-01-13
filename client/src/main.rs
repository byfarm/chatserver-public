use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json;

// make structures for sending datat between client and server
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ServerToClient {
    status: u16,
    messages: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientToServer {
    status: u16,
    action: String,
    messages: Vec<String>,
    ipaddress: String,
}

// for passing cli arugments
#[derive(Parser)]
struct Cli {
    /// the command to run (`send` or `recieve`)
    cmd: String,

    /// the message to send with the `send` command
    message: Option<String>,
}

// add eof to export streams
const END_MESSAGE: &str = "\r\n\r\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // get the cli aruments into args
    let args = Cli::parse();

    // match the cli arguments
    match args.cmd.as_str() {
        "recieve" => match recieve() {
            Ok(_) => {
                println!("success")
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        },
        "send" => match send(args.message) {
            Ok(_) => {
                println!("success")
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        },
        _ => println!("Put in a valid command dumbass"),
    }
    Ok(())
}

fn send(message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // get the message from the option type
    let message_contents = message.expect("Send a Message, Dumbass.");

    // serialize the message to the sending json obj
    let sendee = ClientToServer {
        status: 200,
        action: "send".to_owned(),
        messages: vec![message_contents],
        ipaddress: get_ipaddress(),
    };

    // send the message and get the response
    let response = communicate(sendee)?;

    // print the server response
    println!("{}", response.messages.first().unwrap());
    Ok(())
}

fn get_ipaddress() -> String {
    // gets the ipaddress based on a command
    let output = Command::new("bash")
        .arg("-c")
        .arg("hostname -I | awk '{print $1}'")
        .output()
        .expect("ipaddress command failed.\n");

    let ipaddress = String::from_utf8_lossy(&output.stdout).trim().to_string();

    return ipaddress
}

fn recieve() -> Result<(), Box<dyn std::error::Error>> {

    let sendee = ClientToServer {
        status: 200,
        action: "recieve".to_owned(),
        messages: Vec::new(),
        ipaddress: get_ipaddress(),
    };

    // send the message and get the response
    let response = communicate(sendee)?;

    println!(
        "{}",
        response
            .messages
            .first()
            .expect("You have no new messages, you loser.")
    );
    Ok(())
}

fn communicate(sendee: ClientToServer) -> Result<ServerToClient, Box<dyn std::error::Error>> {
    let address = env::var("ADDRESS").unwrap_or_else(|_| "10.0.0.205:80".to_string());

    // create the tcp communication
    let mut stream = TcpStream::connect(address.to_string())?;

    // convert to json
    let mut message = serde_json::to_string(&sendee)?;

    // add the eof to the message
    message.push_str(END_MESSAGE);

    // write the message to the server
    stream.write_all(message.as_bytes())?;

    // make space for the response
    let mut response = String::new();
    let mut buf_reader = BufReader::new(stream);

    // read the response till the eof
    while let Ok(bytes_read) = buf_reader.read_line(&mut response) {
        if bytes_read == 0 || response.ends_with(END_MESSAGE) {
            break;
        }
    }

    // convert the server response to a parsable structure
    let server_response: ServerToClient = serde_json::from_str(&response.to_owned())?;

    Ok(server_response)
}
