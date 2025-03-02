use std::io::prelude::*;
use std::io::BufReader;

const BULK_STRING: char = '$';
const SIMPLE_STRING: char = '+';
const ERROR: char = '-';
const INTEGER: char = ':';
const ARRAY: char = '*';
const LINE_TERMINATORS: &str = "\r\n";

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
                write!(buf, "{s}{LINE_TERMINATORS}")
            }
            RespData::Error(e) => {
                buf.write_all(&[ERROR as u8])?;
                write!(buf, "ERR {e}{LINE_TERMINATORS}")
            }
            RespData::Integer(n) => {
                buf.write_all(&[INTEGER as u8])?;
                write!(buf, "{n}{LINE_TERMINATORS}")
            }
            RespData::BulkString(s) => {
                buf.write_all(&[BULK_STRING as u8])?;
                write!(
                    buf,
                    "{len}{LINE_TERMINATORS}{s}{LINE_TERMINATORS}",
                    len = s.len()
                )
            }
            RespData::Array(arr) => {
                buf.write_all(&[ARRAY as u8])?;
                write!(buf, "{len}{LINE_TERMINATORS}", len = arr.len())?;
                for item in arr {
                    item.write(buf)?;
                }
                Ok(())
            }
            RespData::Null => {
                write!(buf, "$-1{LINE_TERMINATORS}")
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
    use crate::util::assert_format_repr;

    #[test]
    fn test_simple_string_write_to_buf() {
        assert_format_repr(&RespData::SimpleString("OK".into()), b"+OK\r\n");
    }

    #[test]
    fn test_error_write_to_buf() {
        assert_format_repr(
            &RespData::Error("Invalid command".to_string()),
            b"-ERR Invalid command\r\n",
        )
    }

    #[test]
    fn test_integer_write_to_buf() {
        assert_format_repr(&RespData::Integer(123), b":123\r\n");
        assert_format_repr(&RespData::Integer(-123), b":-123\r\n");
    }

    #[test]
    fn test_bulk_string_write_to_buf() {
        assert_format_repr(
            &RespData::BulkString("hello".to_string()),
            b"$5\r\nhello\r\n",
        );
        assert_format_repr(&RespData::BulkString("".to_string()), b"$0\r\n\r\n");
    }

    #[test]
    fn test_array_write_to_buf() {
        assert_format_repr(
            &RespData::Array(vec![
                RespData::SimpleString("OK".to_string()),
                RespData::Integer(123),
                RespData::BulkString("hello".to_string()),
            ]),
            b"*3\r\n+OK\r\n:123\r\n$5\r\nhello\r\n",
        );
        assert_format_repr(&RespData::Array(vec![]), b"*0\r\n");
    }

    #[test]
    fn test_null_write_to_buf() {
        assert_format_repr(&RespData::Null, b"$-1\r\n");
    }
}
