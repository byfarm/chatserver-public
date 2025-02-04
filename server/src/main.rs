use dotenv::dotenv;
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlite::{Connection, State};

use serde::{Deserialize, Serialize};
use serde_json;

// remake the structures for sending data between the two
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

// add eof to each message
const END_MESSAGE: &str = "\r\n\r\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load the enviornment variables in .env
    dotenv().ok();

    // get the ip address
    let ip_address =
        env::var("ADDRESS").expect("Add `export ADDRESS=<wanted address> to .env file");

    println!("listening on port {}", ip_address);
    // set the tcp listener (ownership thing with &ip_address)
    let listener = TcpListener::bind(&ip_address).unwrap();

    // gracefully handle the incoming streams so ther server does not crash
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                if let Err(e) = handle_stream(&stream) {
                    eprintln!("Error handling stream {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error getting stream result {}", e);
                continue;
            }
        }
    }
    Ok(())
}

fn handle_stream(stream: &TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // print the request
    let body = handle_request(&stream)?;

    // send http success
    send_http_success(&stream, body.to_string())?;
    Ok(())
}

fn handle_request(stream: &TcpStream) -> Result<String, Box<dyn std::error::Error>> {
    let parsed_request = recieve_request(stream)?;

    let response: String = match parsed_request.action.as_str() {
        "recieve" => read_message_from_db(),
        "createuser" => match find_user(parsed_request.ipaddress.clone()) {
            Ok(user_pk) => {
                if user_pk == 0 {
                    println!("{}", parsed_request.ipaddress);
                    println!("{}", user_pk);
                    let username = create_user(parsed_request.ipaddress, parsed_request.messages)?;
                    format!("{} Created.", username)
                } else {
                    "User already in database.".to_string()
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                "Error in creating user.".to_string()
            }
        },
        "send" => match find_user(parsed_request.ipaddress) {
            Ok(user_pk) => {
                if user_pk != 0 {
                    write_new_messages_to_db(parsed_request.messages, user_pk);
                    "Success!".to_string()
                } else {
                    "Must run `createuser` before you can send messages.".to_string()
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                "Error in finding user".to_string()
            }
        },
        _ => "Invalid command. Only can run `send`, `recieve`, or `createuser`".to_string(),
    };
    Ok(response)
}

fn recieve_request(stream: &TcpStream) -> Result<ClientToServer, Box<dyn std::error::Error>> {
    // create the buffer reader
    let mut buf_reader = BufReader::new(stream);

    // create the string the body goes into
    let mut body = String::new();

    // read the contents into the string
    while let Ok(bytes_read) = buf_reader.read_line(&mut body) {
        // once read end character then break
        if bytes_read == 0 || body.ends_with("\r\n\r\n") {
            break;
        }
    }

    // deserialze the request
    let request: ClientToServer = serde_json::from_str(&body.to_owned())?;
    Ok(request)
}

fn find_user(ipaddress: String) -> Result<i64, Box<dyn std::error::Error>> {
    // finds the username in the database from the ip address
    let connection = Connection::open("chat.db")?;
    let query = format!("SELECT pk FROM users WHERE ipaddress = '{}' LIMIT 1;", ipaddress);
    let mut user_pk: i64 = 0;
    connection.iterate(query, |row| {
        for (_, val) in row {
            user_pk = val.unwrap().to_string().parse::<i64>().unwrap()
        }
        true
    }).unwrap();

    return Ok(user_pk);
}

fn create_user(
    ipaddress: String,
    username: Vec<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let connection = Connection::open("chat.db")?;

    let user = username.join("");

    // sql injection!!
    let query = format!(
        "INSERT INTO users (ipaddress, username) VALUES('{}', '{}');",
        ipaddress.to_string(),
        user,
    );

    connection.execute(query)?;
    return Ok(user);
}

fn write_new_messages_to_db(messages: Vec<String>, user_pk: i64) {
    let connection = Connection::open("chat.db").unwrap();
    for msg in messages {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let query = format!(
            "
        INSERT INTO messages (message, timestamp, read, user) VALUES ('{}', {}, 0, {});
        ",
            msg, time, user_pk
        );
        connection.execute(query).unwrap();
    }
}

fn read_message_from_db() -> String {
    let connection = Connection::open("chat.db").unwrap();
    let query = "SELECT username, message FROM messages JOIN users ON messages.user = users.pk;";
    let mut statement = connection.prepare(query).unwrap();

    let mut large_body = String::new();
    while let Ok(State::Row) = statement.next() {
        let message = &statement.read::<String, _>("message").unwrap();
        let user = &statement.read::<String, _>("username").unwrap();
        let content = format!("{}: {}\n", user, message);
        large_body.push_str(content.as_str());
    }
    return large_body;
}

fn send_http_success(
    mut stream: &TcpStream,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // make the request message
    let response = ServerToClient {
        status: 200,
        messages: vec![body.to_owned()],
    };

    // turn the response to a string
    let mut response_string = serde_json::to_string(&response)?;
    response_string.push_str(END_MESSAGE);

    // send the response
    stream.write_all(response_string.as_bytes()).unwrap();
    Ok(())
}
