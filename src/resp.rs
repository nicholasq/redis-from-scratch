use std::io::prelude::*;
use std::io::BufReader;

const BULK_STRING: char = '$';
const SIMPLE_STRING: char = '+';
const ERROR: char = '-';
const INTEGER: char = ':';
const ARRAY: char = '*';
const LINE_TERMINATORS: &[u8] = b"\r\n";

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RespData {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespData>),
    Null,
}

impl RespData {
    pub fn write(&self, buf: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            RespData::SimpleString(s) => {
                buf.write_all(&[SIMPLE_STRING as u8])?;
                buf.write_all(s.as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                Ok(())
            }
            RespData::Error(e) => {
                buf.write_all(&[ERROR as u8])?;
                buf.write_all(b"ERR ")?;
                buf.write_all(e.as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                Ok(())
            }
            RespData::Integer(n) => {
                buf.write_all(&[INTEGER as u8])?;
                buf.write_all(n.to_string().as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                Ok(())
            }
            RespData::BulkString(s) => {
                buf.write_all(&[BULK_STRING as u8])?;
                buf.write_all(s.len().to_string().as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                buf.write_all(s.as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                Ok(())
            }
            RespData::Array(arr) => {
                buf.write_all(&[ARRAY as u8])?;
                buf.write_all(arr.len().to_string().as_bytes())?;
                buf.write_all(LINE_TERMINATORS)?;
                for item in arr {
                    item.write(buf)?;
                }
                Ok(())
            }
            RespData::Null => {
                buf.write_all(b"$-1")?;
                buf.write_all(LINE_TERMINATORS)?;
                Ok(())
            }
        }
    }
}

pub struct Resp<R: Read> {
    reader: BufReader<R>,
    pub raw_data: String,
    lines: Vec<String>,
}

impl<R: Read> Resp<R> {
    pub fn new(input: R) -> Self {
        Resp {
            reader: BufReader::new(input),
            raw_data: String::new(),
            lines: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<RespData, std::io::Error> {
        let line = self.read_line()?;

        if line.starts_with(SIMPLE_STRING) {
            let line = self.read_line()?;
            return Ok(RespData::SimpleString(line));
        }

        if line.starts_with(BULK_STRING) {
            let line = self.read_line()?;
            return Ok(RespData::BulkString(line));
        }

        if line.starts_with(ARRAY) {
            let num = self.read_integer(&line[1..])?;
            let mut array = Vec::with_capacity(num as usize);
            for _ in 0..num {
                array.push(self.read()?);
            }
            return Ok(RespData::Array(array));
        }

        if line.starts_with(INTEGER) {
            let num = self.read_integer(&line[1..])?;
            return Ok(RespData::Integer(num));
        }

        Ok(RespData::Error("Unknown error".to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_simple_string_write_to_buf() {
        let simple_string = RespData::SimpleString("OK".to_string());
        let mut buffer = Cursor::new(Vec::new());
        simple_string.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"+OK\r\n");
    }

    #[test]
    fn test_error_write_to_buf() {
        let error = RespData::Error("Invalid command".to_string());
        let mut buffer = Cursor::new(Vec::new());
        error.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"-ERR Invalid command\r\n");
    }

    #[test]
    fn test_integer_write_to_buf() {
        let integer = RespData::Integer(123);
        let mut buffer = Cursor::new(Vec::new());
        integer.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b":123\r\n");

        let negative = RespData::Integer(-123);
        let mut buffer = Cursor::new(Vec::new());
        negative.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b":-123\r\n");
    }

    #[test]
    fn test_bulk_string_write_to_buf() {
        let bulk_string = RespData::BulkString("hello".to_string());
        let mut buffer = Cursor::new(Vec::new());
        bulk_string.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"$5\r\nhello\r\n");

        let empty_string = RespData::BulkString("".to_string());
        let mut buffer = Cursor::new(Vec::new());
        empty_string.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"$0\r\n\r\n");
    }

    #[test]
    fn test_array_write_to_buf() {
        let array = RespData::Array(vec![
            RespData::SimpleString("OK".to_string()),
            RespData::Integer(123),
            RespData::BulkString("hello".to_string()),
        ]);
        let mut buffer = Cursor::new(Vec::new());
        array.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"*3\r\n+OK\r\n:123\r\n$5\r\nhello\r\n");

        let empty_array = RespData::Array(vec![]);
        let mut buffer = Cursor::new(Vec::new());
        empty_array.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"*0\r\n");
    }

    #[test]
    fn test_null_write_to_buf() {
        let null = RespData::Null;
        let mut buffer = Cursor::new(Vec::new());
        null.write(&mut buffer).unwrap();
        assert_eq!(buffer.into_inner(), b"$-1\r\n");
    }
}
