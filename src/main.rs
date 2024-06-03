use std::{env, fs};

// Available if you need it!
use serde_bencode;
use serde_json;
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



struct Torrent {
    tracker_url: Vec<u8>,
    info: TorrentInfo
}

struct TorrentInfo {
    length: i64,
    name: String,
    piece_length: i64,
    pieces: Vec<u8>
}

fn get_torrent(file_name: &str) -> anyhow::Result<Torrent> {
    let data: Vec<u8> = fs::read(file_name).expect("file couldn't be read");

    let result: serde_bencode::value::Value = serde_bencode::from_bytes(&*data).expect("failed to parse file data");
    match result {
        serde_bencode::value::Value::Dict(dict) => {
            let mut torrent: Torrent = Torrent {
                tracker_url: vec![],
                info: TorrentInfo {
                    length: 0,
                    name: "".to_string(),
                    piece_length: 0,
                    pieces: vec![],
                }
            };

            let announce_value = dict.get("announce".as_bytes()).expect("key 'announce' should exist in the dict");
            match announce_value {
                serde_bencode::value::Value::Bytes(serde_bytes) => {
                    torrent.tracker_url = serde_bytes.clone();
                }

                _ => {
                    panic!("announce should always be a byte stream")
                }
            }

            let info_value = dict.get("info".as_bytes()).expect("key 'info' should exist in the dict");
            match info_value {
                serde_bencode::value::Value::Dict(info_dict) => {
                    if let serde_bencode::value::Value::Int(length_in_dict) = info_dict.get("length".as_bytes()).expect("key 'length' should be part of info dictionary") {
                        torrent.info.length = *length_in_dict;
                    }
                }

                _ => {
                    panic!("info should be a dictionary in torrent file")
                }
            }

            Ok(torrent)
        }

        _ => {
            panic!("file data should represent a dictionary");
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
        let file_name = &args[2];
        let torrent = get_torrent(file_name).expect("Torrent couldn't be obtained");
        println!("Tracker URL: {}", String::from_utf8(torrent.tracker_url).expect("tracker URL should be a string"));
        println!("Length: {}", torrent.info.length);
    } else {
        println!("unknown command: {}", args[1])
    }
}

// lli543e9:blueberryee
// lli4eei5ee
