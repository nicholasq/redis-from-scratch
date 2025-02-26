use std::io::BufWriter;
use std::str;
use std::{io::BufReader, net};

use resp::RespData;

mod resp;

const ADDR: &str = "0.0.0.0:6379";

fn main() {
    let listener = net::TcpListener::bind(ADDR).unwrap();

    println!("Listening on {}", ADDR);

    for stream in listener.incoming() {
        println!("Connection established");
        let stream = stream.unwrap();
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);

        let mut resp = resp::Resp::new(reader);
        let data = resp.read().unwrap();

        println!("Raw data: {:?}", resp.raw_data);
        println!("Parsed data: {:?}", data);

        RespData::SimpleString("OK".to_string())
            .write(&mut writer)
            .unwrap();
    }
}
