use std::collections::HashMap;
use std::io::BufWriter;
use std::str;
use std::{io::BufReader, net};

use handler::CommandHandler;

mod handler;
mod resp;
mod util;

const ADDR: &str = "0.0.0.0:6379";

fn main() {
    let listener = net::TcpListener::bind(ADDR).unwrap();

    println!("Listening on {}", ADDR);

    let db = HashMap::new();
    let mut cmd_handler = CommandHandler::from(db);

    for stream in listener.incoming() {
        println!("Connection established");
        let stream = stream.unwrap();
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);

        let mut resp = resp::Resp::new(reader);
        let data = resp.read().unwrap();

        println!("Raw data: {:?}", resp.raw_data);
        println!("Parsed data: {:?}", data);

        let response = cmd_handler.handle(&data);
        println!("Response: {:?}", response);
        response.write(&mut writer).unwrap();
    }
}
