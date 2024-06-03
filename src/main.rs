use std::collections::HashMap;
use serde_json;
use std::env;
use hex::{decode, encode};
use serde::{Deserialize, Serialize};

// Available if you need it!
use serde_bencode;
use serde_json::Value;

fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<Value> {
    let bencoded_value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value).expect("encoded value is invalid");
    return transform_bencoded_value_to_json_value(bencoded_value)
}

fn transform_bencoded_value_to_json_value(bencoded_value: serde_bencode::value::Value) -> anyhow::Result<serde_json::Value> {
    match bencoded_value {
        serde_bencode::value::Value::Bytes(b) => {
            let string = String::from_utf8(b).expect("should be a valid UTF-8 string");
            Ok(Value::String(string))
        }

        serde_bencode::value::Value::Int(i) => {
            Ok(Value::Number(serde_json::Number::from(i)))
        }

        serde_bencode::value::Value::List(list) => {
            let mut answer: Vec<Value> = vec![];
            for element in list.into_iter() {
                answer.push(transform_bencoded_value_to_json_value(element).expect("Individual element of list should be a valid encoded value"))
            }

            Ok(Value::Array(answer))
        }

        serde_bencode::value::Value::Dict(dict) => {
            let mut answer: serde_json::value::Map<String, Value> = Default::default();
            for (key, value) in dict.into_iter() {
                let modified_key = String::from_utf8(key).expect("key should be a valid string in dict");
                answer.insert(modified_key, transform_bencoded_value_to_json_value(value).expect("value should be a valid value in dict"));
            }

            Ok(Value::Object(answer))
        }
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value).expect("encoded value must be valid");
        println!("{}", decoded_value.to_string());
    } else if command == "info" {

    } else {
        println!("unknown command: {}", args[1])
    }
}

// lli543e9:blueberryee
// lli4eei5ee
