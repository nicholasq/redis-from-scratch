use crate::resp::RespData;
use std::collections::HashMap;

pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
}

pub struct CommandHandler {
    db: HashMap<String, RedisValue>,
}

impl CommandHandler {
    pub fn from(db: HashMap<String, RedisValue>) -> Self {
        Self { db }
    }

    pub fn handle(&mut self, resp: &RespData) -> RespData {
        let cmd = match resp {
            RespData::SimpleString(str) => str,
            RespData::BulkString(str) => str,
            RespData::Array(arr) => match arr.first() {
                Some(RespData::BulkString(str)) => str,
                Some(RespData::SimpleString(str)) => str,
                _ => {
                    return RespData::Error("Invalid command".to_string());
                }
            },
            _ => {
                return RespData::Error("Invalid command".to_string());
            }
        };

        match cmd.to_uppercase().as_str() {
            "PING" => self.ping(),
            "SET" => self.set(resp),
            "GET" => self.get(resp),
            "HSET" => self.hset(resp),
            "HGET" => self.hget(resp),
            "HGETALL" => self.hgetall(resp),
            _ => RespData::Error("Invalid command".to_string()),
        }
    }

    fn ping(&mut self) -> RespData {
        RespData::SimpleString("PONG".to_string())
    }

    fn set(&mut self, resp: &RespData) -> RespData {
        let RespData::Array(arr) = resp else {
            return RespData::Error("syntax error".to_string());
        };
        if arr.len() > 3 {
            return RespData::Error("syntax error".to_string());
        };
        let [_, RespData::BulkString(key), RespData::BulkString(value)] = arr.as_slice() else {
            return RespData::Error("wrong number of arguments for 'set' command".to_string());
        };
        self.db
            .insert(key.clone(), RedisValue::String(value.clone()));
        RespData::SimpleString("OK".to_string())
    }

    fn get(&mut self, resp: &RespData) -> RespData {
        let RespData::Array(arr) = resp else {
            return RespData::Error("syntax error".to_string());
        };

        if arr.len() != 2 {
            return RespData::Error("wrong number of arguments for 'get' command".to_string());
        }

        let [_, RespData::BulkString(key)] = arr.as_slice() else {
            return RespData::Error("syntax error".to_string());
        };

        self.db
            .get(key)
            .map_or(RespData::Null, |value| match value {
                RedisValue::String(value) => RespData::BulkString(value.clone()),
                _ => RespData::Null,
            })
    }

    fn hset(&mut self, resp: &RespData) -> RespData {
        let RespData::Array(arr) = resp else {
            return RespData::Error("wrong number of arguments for 'hset' command".to_string());
        };

        if arr.len() < 4 || arr.len() % 2 != 0 {
            return RespData::Error("wrong number of arguments for 'hset' command".to_string());
        }

        let RespData::BulkString(hash_key) = &arr[0] else {
            return RespData::Error("wrong number of arguments for 'hset' command".to_string());
        };
        let pairs = &arr[1..];

        let hash_map = match self.db.get_mut(hash_key) {
            Some(RedisValue::Hash(map)) => map,
            None => {
                self.db
                    .insert(hash_key.clone(), RedisValue::Hash(HashMap::new()));
                if let RedisValue::Hash(map) = self.db.get_mut(hash_key).unwrap() {
                    map
                } else {
                    unreachable!()
                }
            }
            _ => {
                return RespData::Error("Key exists but value is not a map".to_string());
            }
        };

        let mut new_fields_count = 0;

        for pair in pairs.chunks_exact(2) {
            let field = match &pair[0] {
                RespData::BulkString(field) => field,
                _ => return RespData::Error("Invalid field type".to_string()),
            };
            let value = match &pair[1] {
                RespData::BulkString(value) => value,
                _ => return RespData::Error("Invalid value type".to_string()),
            };
            let is_new = !hash_map.contains_key(field);
            hash_map.insert(field.clone(), value.clone());
            if is_new {
                new_fields_count += 1;
            }
        }

        RespData::Integer(new_fields_count)
    }

    fn hget(&mut self, resp: &RespData) -> RespData {
        let RespData::Array(arr) = resp else {
            return RespData::Error("wrong number of arguments for 'hget' command".to_string());
        };

        if arr.len() != 3 {
            return RespData::Error("wrong number of arguments for 'hget' command".to_string());
        }

        let (RespData::BulkString(hash_key), RespData::BulkString(field)) = (&arr[1], &arr[2])
        else {
            return RespData::Error("wrong number of arguments for 'hget' command".to_string());
        };

        match self.db.get(hash_key) {
            Some(RedisValue::Hash(map)) => map
                .get(field)
                .map_or(RespData::Null, |value| RespData::BulkString(value.clone())),
            Some(_) => RespData::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            ),
            None => RespData::Null,
        }
    }

    fn hgetall(&mut self, resp: &RespData) -> RespData {
        let RespData::Array(arr) = resp else {
            return RespData::Error("wrong number of arguments for 'hgetall' command".to_string());
        };

        if arr.len() < 2 {
            return RespData::Error("wrong number of arguments for 'hgetall' command".to_string());
        }

        let hash_key = match &arr[1] {
            RespData::BulkString(hash) => hash,
            _ => {
                panic!("'hgetall' command arg was not a bulk string: {:?}", arr[1]);
            }
        };

        match self.db.get(hash_key) {
            Some(RedisValue::Hash(map)) => {
                let mut result = Vec::new();
                for (field, value) in map {
                    result.push(RespData::BulkString(field.clone()));
                    result.push(RespData::BulkString(value.clone()));
                }
                RespData::Array(result)
            }
            _ => RespData::Array(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::RespData;
    use std::collections::{HashMap, HashSet};

    fn create_empty_handler() -> CommandHandler {
        CommandHandler::from(HashMap::new())
    }

    #[test]
    fn test_ping() {
        let mut handler = create_empty_handler();

        let result = handler.ping();

        assert_eq!(result, RespData::SimpleString("PONG".to_string()));
    }

    #[test]
    fn test_set() {
        let mut handler = create_empty_handler();

        let test_cases = [
            (
                "Valid SET command",
                RespData::Array(vec![
                    RespData::BulkString("SET".to_string()),
                    RespData::BulkString("key1".to_string()),
                    RespData::BulkString("value1".to_string()),
                ]),
                RespData::SimpleString("OK".to_string()),
            ),
            (
                "Not enough arguments",
                RespData::Array(vec![
                    RespData::BulkString("SET".to_string()),
                    RespData::BulkString("key1".to_string()),
                ]),
                RespData::Error("wrong number of arguments for 'set' command".to_string()),
            ),
            (
                "Too many arguments",
                RespData::Array(vec![
                    RespData::BulkString("SET".to_string()),
                    RespData::BulkString("key1".to_string()),
                    RespData::BulkString("value1".to_string()),
                    RespData::BulkString("value2".to_string()),
                ]),
                RespData::Error("syntax error".to_string()),
            ),
        ];

        for (name, input, expected_output) in test_cases {
            let result = handler.set(&input);
            assert_eq!(result, expected_output, "{}", name);
        }
    }

    #[test]
    fn test_get() {
        let mut handler = create_empty_handler();

        handler.db.insert(
            "existing_key".to_string(),
            RedisValue::String("existing_value".to_string()),
        );

        let test_cases = [
            (
                "Valid GET for existing key",
                RespData::Array(vec![
                    RespData::BulkString("GET".to_string()),
                    RespData::BulkString("existing_key".to_string()),
                ]),
                RespData::BulkString("existing_value".to_string()),
            ),
            (
                "Valid GET for non-existing key",
                RespData::Array(vec![
                    RespData::BulkString("GET".to_string()),
                    RespData::BulkString("non_existing_key".to_string()),
                ]),
                RespData::Null,
            ),
            (
                "Not enough arguments",
                RespData::Array(vec![RespData::BulkString("GET".to_string())]),
                RespData::Error("wrong number of arguments for 'get' command".to_string()),
            ),
            (
                "Too many arguments",
                RespData::Array(vec![
                    RespData::BulkString("GET".to_string()),
                    RespData::BulkString("key1".to_string()),
                    RespData::BulkString("key2".to_string()),
                ]),
                RespData::Error("wrong number of arguments for 'get' command".to_string()),
            ),
        ];

        for (name, input, expected_output) in test_cases {
            let result = handler.get(&input);
            assert_eq!(result, expected_output, "{}", name);
        }
    }

    #[test]
    fn test_hset() {
        let mut handler = create_empty_handler();

        let mut initial_hash = HashMap::new();
        initial_hash.insert("field1".to_string(), "value1".to_string());
        handler
            .db
            .insert("existing_hash".to_string(), RedisValue::Hash(initial_hash));
        handler.db.insert(
            "string_key".to_string(),
            RedisValue::String("string_value".to_string()),
        );

        let test_cases = [
            (
                "Valid HSET create a new hash",
                RespData::Array(vec![
                    RespData::BulkString("HSET".to_string()),
                    RespData::BulkString("new_hash".to_string()),
                    RespData::BulkString("field1".to_string()),
                    RespData::BulkString("value1".to_string()),
                ]),
                RespData::Integer(1), // New field
            ),
            (
                "Valid HSET adding new field to existing hash",
                RespData::Array(vec![
                    RespData::BulkString("HSET".to_string()),
                    RespData::BulkString("existing_hash".to_string()),
                    RespData::BulkString("field2".to_string()),
                    RespData::BulkString("value2".to_string()),
                ]),
                RespData::Integer(1), // New field
            ),
            (
                "Valid HSET updating existing field",
                RespData::Array(vec![
                    RespData::BulkString("HSET".to_string()),
                    RespData::BulkString("existing_hash".to_string()),
                    RespData::BulkString("field1".to_string()),
                    RespData::BulkString("new_value".to_string()),
                ]),
                RespData::Integer(0), // Existing field
            ),
            (
                "Not enough arguments",
                RespData::Array(vec![
                    RespData::BulkString("HSET".to_string()),
                    RespData::BulkString("hash".to_string()),
                    RespData::BulkString("field".to_string()),
                ]),
                RespData::Error("wrong number of arguments for 'hset' command".to_string()),
            ),
            (
                "Invalid key value pair arguments",
                RespData::Array(vec![
                    RespData::BulkString("HSET".to_string()),
                    RespData::BulkString("hash".to_string()),
                    RespData::BulkString("field1".to_string()),
                    RespData::BulkString("value1".to_string()),
                    RespData::BulkString("field2".to_string()),
                ]),
                RespData::Error("wrong number of arguments for 'hset' command".to_string()),
            ),
        ];

        for (name, input, expected_output) in test_cases {
            let result = handler.hset(&input);
            assert_eq!(result, expected_output, "{}", name);
        }
    }

    #[test]
    fn test_hget() {
        let mut handler = create_empty_handler();

        // Set up some test data in the DB
        let mut test_hash = HashMap::new();
        test_hash.insert("existing_field".to_string(), "field_value".to_string());
        handler
            .db
            .insert("existing_hash".to_string(), RedisValue::Hash(test_hash));
        handler.db.insert(
            "string_key".to_string(),
            RedisValue::String("string_value".to_string()),
        );

        // Test cases with different inputs
        let test_cases = [
            (
                "Valid HGET for existing hash and field",
                RespData::Array(vec![
                    RespData::BulkString("HGET".to_string()),
                    RespData::BulkString("existing_hash".to_string()),
                    RespData::BulkString("existing_field".to_string()),
                ]),
                RespData::BulkString("field_value".to_string()),
            ),
            (
                "Valid HGET for existing hash but non-existing field",
                RespData::Array(vec![
                    RespData::BulkString("HGET".to_string()),
                    RespData::BulkString("existing_hash".to_string()),
                    RespData::BulkString("non_existing_field".to_string()),
                ]),
                RespData::Null,
            ),
            (
                "HGET for non-existing hash",
                RespData::Array(vec![
                    RespData::BulkString("HGET".to_string()),
                    RespData::BulkString("non_existing_hash".to_string()),
                    RespData::BulkString("field".to_string()),
                ]),
                RespData::Null,
            ),
            (
                "Not enough arguments",
                RespData::Array(vec![
                    RespData::BulkString("HGET".to_string()),
                    RespData::BulkString("hash".to_string()),
                ]),
                RespData::Error("wrong number of arguments for 'hget' command".to_string()),
            ),
        ];

        for (name, input, expected_output) in test_cases {
            let result = handler.hget(&input);
            assert_eq!(result, expected_output, "{}", name);
        }
    }

    #[test]
    fn test_hgetall() {
        let mut handler = create_empty_handler();

        let mut hash_map = HashMap::new();
        hash_map.insert("field1".to_string(), "value1".to_string());
        hash_map.insert("field2".to_string(), "value2".to_string());
        handler
            .db
            .insert("hash_key".to_string(), RedisValue::Hash(hash_map));

        handler.db.insert(
            "string_key".to_string(),
            RedisValue::String("some_string".to_string()),
        );

        let test_cases = [
            (
                "Valid HGETALL for existing hash",
                RespData::Array(vec![
                    RespData::BulkString("HGETALL".to_string()),
                    RespData::BulkString("hash_key".to_string()),
                ]),
                // Expected result is an array with field-value pairs
                // Note: we can't predict the exact order of fields due to HashMap
                RespData::Array(vec![
                    RespData::BulkString("field1".to_string()),
                    RespData::BulkString("value1".to_string()),
                    RespData::BulkString("field2".to_string()),
                    RespData::BulkString("value2".to_string()),
                ]),
            ),
            (
                "HGETALL for non-existing hash",
                RespData::Array(vec![
                    RespData::BulkString("HGETALL".to_string()),
                    RespData::BulkString("non_existing_key".to_string()),
                ]),
                RespData::Array(vec![]),
            ),
            (
                "Not enough arguments",
                RespData::Array(vec![RespData::BulkString("HGETALL".to_string())]),
                RespData::Error("wrong number of arguments for 'hgetall' command".to_string()),
            ),
        ];

        for (name, input, expected_result) in test_cases {
            let result = handler.hgetall(&input);
            match (result, expected_result) {
                (RespData::Array(res), RespData::Array(exp)) => {
                    let result_hashset: HashSet<RespData> = res.into_iter().collect();
                    assert_eq!(result_hashset, exp.into_iter().collect(), "{}", name);
                }
                (RespData::Error(res), RespData::Error(exp)) => {
                    assert_eq!(res, exp, "{}", name);
                }
                _ => {
                    panic!("Unexpected result for {}", name);
                }
            }
        }
    }
}
