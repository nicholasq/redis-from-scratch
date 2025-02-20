use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType {
    SimpleString(String) = '+' as u8,
    Error = '-' as u8,
    Integer(i64) = ':' as u8,
    BulkString(String) = '$' as u8,
    Array(Vec<DataType>) = '*' as u8,
}

pub struct Resp<'a> {
    reader: BufReader<&'a TcpStream>,
    pub raw_data: String,
    lines: Vec<String>,
}

impl<'a> Resp<'a> {
    pub fn new(reader: BufReader<&'a TcpStream>) -> Self {
        Resp {
            reader,
            raw_data: String::new(),
            lines: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<DataType, std::io::Error> {
        let line = self.read_line()?;

        if line.starts_with('$') {
            let line = self.read_line()?;
            return Ok(DataType::SimpleString(line));
        }

        if line.starts_with('*') {
            let num = self.read_integer(&line[1..])?;
            let mut array = Vec::with_capacity(num as usize);
            for _ in 0..num {
                array.push(self.read()?);
            }
            return Ok(DataType::Array(array));
        }

        if line.starts_with(':') {
            let num = self.read_integer(&line[1..])?;
            return Ok(DataType::Integer(num));
        }

        Ok(DataType::Error)
    }

    pub fn read_line(&mut self) -> Result<String, std::io::Error> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        self.raw_data.push_str(&line);
        self.lines.push(line);
        Ok(self.lines.last().unwrap().trim().to_string())
    }

    pub fn read_integer(&mut self, line: &str) -> Result<i64, std::io::Error> {
        let num = line
            .trim()
            .parse::<i64>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(num)
    }
}
