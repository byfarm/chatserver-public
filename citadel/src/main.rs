use std::env;
use dotenv::dotenv;
use std::{net::{TcpStream, TcpListener}, io::{prelude::*, BufReader}};
use std::time::{SystemTime, UNIX_EPOCH};
// use sqlite::{Connection, State};
use serde::{Deserialize, Serialize};
use serde_json;

use heapless::String as enString;
use esp_idf_hal::{
    peripherals::Peripherals,
};
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};
use embedded_svc::wifi::{ClientConfiguration, Configuration};

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
}

// add eof to each message
const END_MESSAGE: &str = "\r\n\r\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();

    // load the enviornment variables in .env
    dotenv().ok();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    // let ssid_env = env::var("WIFI_NAME").expect("wifi name not set");
    // let password_env = env::var("WIFI_PASS").expect("wifi password not set");
    let ssid: enString<32> = enString::try_from("HappyFamily").unwrap();
    let password: enString<64> = enString::try_from("wifi_password").unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid,
        password,
        ..Default::default()
    })).unwrap();


    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
    }
    log::info!("Should be connected now");

    // get the ip address
    let ip_address = "0.0.0.0:80";

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

fn setup_wifi() {
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
        "recieve" => /* read_message_from_db() */ {
            log::info!("Recieved Response");
            "Hello".to_string()},
        "send" => {
            write_new_messages_to_db(parsed_request.messages);
            "Success!".to_string()
        }
        _ => "Invalid command. Only can run `send` or `recieve`".to_string(),
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

fn write_new_messages_to_db(messages: Vec<String>) {
    // let connection = Connection::open("chat.db").unwrap();
    // for msg in messages {
    //     let time = SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs();
    //
    //     let query = format!(
    //         "
    //     INSERT INTO messages (message, timestamp, read) VALUES ('{}', {}, 0);
    //     ",
    //         msg, time
    //     );
    //     connection.execute(query).unwrap();
    // }
}

/* fn read_message_from_db() -> String {
    let connection = Connection::open("chat.db").unwrap();
    let query = "SELECT message FROM messages;";
    let mut statement = connection.prepare(query).unwrap();

    let mut large_body = String::new();
    while let Ok(State::Row) = statement.next() {
        large_body.push_str(&statement.read::<String, _>("message").unwrap());
        large_body.push_str("\n");
    }
    return large_body;
} */

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
