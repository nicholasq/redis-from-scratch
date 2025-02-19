use std::str;
use std::{
    io::{BufReader, Read, Write},
    net,
};

const ADDR: &str = "0.0.0.0:6379";

fn main() {
    let listener = net::TcpListener::bind(ADDR).unwrap();

    println!("Listening on {}", ADDR);

    for stream in listener.incoming() {
        println!("Connection established");
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(&stream);
        let mut buf = vec![];
        let count = reader.read(&mut buf).unwrap();
        let content = str::from_utf8(&buf[0..count]).unwrap();
        println!("{}", content);
        stream.write_all("+OK\r\n".as_bytes()).unwrap();
    }
}
