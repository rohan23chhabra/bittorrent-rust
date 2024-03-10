use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, usize) {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let len_number_string = number_string.chars().count();
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        let len_string = string.chars().count();
        return (serde_json::Value::String(string.to_string()), len_number_string + len_string + 1);
    } else if encoded_value.starts_with("i") {
        // Example: "i-52e" -> 52
        let pos_of_i = encoded_value.find('i').unwrap();
        let pos_of_e = encoded_value.find('e').unwrap();
        // println!("pos_of_i = {}, pos_of_e = {}", pos_of_i, pos_of_e);
        let len = pos_of_e - pos_of_i + 1;
        // println!("length = {}", len);
        let num_str = &encoded_value[1..len - 1];
        // println!("num_str = {}", num_str);
        let number = num_str.parse::<i64>().unwrap();
        // println!("number = {}", number);
        return (serde_json::Value::Number(number.into()), len);
    } else if encoded_value.starts_with("l") && encoded_value.ends_with("e") {
        let len = encoded_value.chars().count();
        // println!("length = {}", len);
        let mut trimmed_value = &encoded_value[1..len - 1];
        let mut i = 0;
        let mut answer_vec: Vec<serde_json::Value> = Vec::new();
        while i < len - 2 {
            // println!("i = {}", i);
            // println!("trimmed_value = {}", trimmed_value);
            let (decoded_value, length) = decode_bencoded_value(trimmed_value);
            // println!("decoded_value = {}, length = {}", decoded_value, length);
            answer_vec.push(decoded_value);
            i += length;
            // println!("updated i = {}", i);
            trimmed_value = &encoded_value[i + 1..len - 1];
            // println!("updated trimmed_value = {}", trimmed_value);
        }

        let answer = serde_json::Value::Array(answer_vec);
        return (answer, len);
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let (decoded_value, _) = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
