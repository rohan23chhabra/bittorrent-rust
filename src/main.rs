use std::{env, fs};
use std::net::Ipv4Addr;

use bytes::Bytes;
use reqwest;
use serde::Serialize;
// Available if you need it!
use serde_bencode;
use serde_json;
use serde_json::Value;
use sha1::{Digest, Sha1};
use urlencoding::encode_binary;

fn decode_string_bencoded_value(encoded_value: &str) -> anyhow::Result<Value> {
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

#[derive(Debug)]
struct Torrent {
    tracker_url: Vec<u8>,
    info: TorrentInfo
}

#[derive(Serialize, Debug)]
struct TorrentInfo {
    length: i64,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    pieces: Vec<String>,
    hash: String
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
                    hash: "".to_string()
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
            torrent.info.hash = hex::encode(Sha1::digest(serde_bencode::to_bytes(info_value).expect("info cannot be serialized")));
            match info_value {
                serde_bencode::value::Value::Dict(info_dict) => {
                    if let serde_bencode::value::Value::Int(length_in_dict) = info_dict.get("length".as_bytes()).expect("key 'length' should exist in info dictionary") {
                        torrent.info.length = *length_in_dict;
                    }

                    if let serde_bencode::value::Value::Int(piece_length) = info_dict.get("piece length".as_bytes()).expect("key 'piece length' should exist in info dictionary") {
                        println!("Piece length = {}", piece_length);
                        torrent.info.piece_length = *piece_length as usize;
                    }

                    if let serde_bencode::value::Value::Bytes(piece_bytes) = info_dict.get("pieces".as_bytes()).expect("key 'pieces' should exist in info dictionary") {
                        let mut ii = 0;
                        while ii < piece_bytes.len() {
                            torrent.info.pieces.push(hex::encode(&piece_bytes[ii..ii + 20]));
                            ii += 20;
                        }
                    }

                    if let serde_bencode::value::Value::Bytes(name) = info_dict.get("name".as_bytes()).expect("key 'name' should exist in info dictionary") {
                        torrent.info.name = String::from_utf8(name.clone()).expect("name should be UTF-8 in info dict");
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

struct TrackerResponse {
    interval: i64,
    peers: Vec<String>
}

fn extract_torrent_info(file_name: &str) -> Torrent {
    let torrent = get_torrent(file_name).expect("Torrent couldn't be obtained");
    println!("Tracker URL: {}", std::str::from_utf8(&torrent.tracker_url)
        .expect("tracker URL isn't a string"));
    println!("Length: {}", torrent.info.length);
    println!("Info Hash: {}", torrent.info.hash);
    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    for piece_hash in &torrent.info.pieces {
        println!("{}", piece_hash);
    }
    return torrent;
}

fn get_peer_data_from_tracker(torrent: &Torrent) -> anyhow::Result<TrackerResponse> {
    let info_hash = encode_binary(hex::decode(&torrent.info.hash)
        .expect("info hash couldn't be decoded").as_slice()).into_owned();
    let peer_id = "00112233445566778899";
    let port = "6881";
    let uploaded = "0";
    let downloaded = "0";
    let left = torrent.info.length;
    let compact = "1";

    let url = std::str::from_utf8(&torrent.tracker_url).expect("Tracker URL should be a valid UTF-8 str");
    let url_with_params = format!("{url}?info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}&downloaded={downloaded}&left={left}&compact={compact}");
    println!("Sending HTTP Request to URL = {}", url_with_params);

    let res = reqwest::blocking::get(url_with_params).expect("Response couldn't be obtained");
    let resp_bytes = res.bytes().expect("response bytes are invalid");
    return extract_tracker_response(resp_bytes);
}

fn extract_tracker_response(resp_bytes: Bytes) -> anyhow::Result<TrackerResponse> {
    let mut tracker_response = TrackerResponse{ interval: 0, peers: vec![] };
    let result: serde_bencode::value::Value = serde_bencode::from_bytes(resp_bytes.as_ref()).expect("resp bytes couldn't be parsed");
    match result {
        serde_bencode::value::Value::Dict(dict) => {
            if let serde_bencode::value::Value::Int(i) = dict.get("interval".as_bytes()).expect("key 'interval' should exist in dict") {
                tracker_response.interval = i.clone();
            }

            if let serde_bencode::value::Value::Bytes(b) = dict.get("peers".as_bytes()).expect("key 'peers' should exist in dict") {
                let mut ii = 0;
                while ii < b.len() {
                    tracker_response.peers.push(parse_ip_address_port(&b[ii..ii + 6]));
                    ii += 6;
                }
            }

            return Ok(tracker_response);
        }

        _ => panic!("Response has to be a bencoded dictionary")
    }
}

fn parse_ip_address_port(b: &[u8]) -> String {
    let ip = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
    let port_bytes = &b[4..6];
    let port = u16::from_be_bytes(port_bytes.try_into().expect(""));
    return format!("{ip}:{port}");
}

fn send_request_and_print_tracker_response(torrent: &Torrent) {
    println!("Inside send_request_and_extract_tracker_response");
    let tracker_response = get_peer_data_from_tracker(&torrent).expect("Tracker Response couldn't be obtained");
    for peer in &tracker_response.peers {
        println!("{peer}");
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_string_bencoded_value(encoded_value).expect("encoded value must be valid");
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        extract_torrent_info(&args[2]);
    } else if command == "peers" {
        let torrent = get_torrent(&args[2]).expect("Torrent couldn't be obtained");
        send_request_and_print_tracker_response(&torrent);
    } else {
        println!("unknown command: {}", args[1])
    }
}

// lli543e9:blueberryee
// lli4eei5ee
