use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<(serde_json::Value, &str), &str> {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_digit(10) {
        if let Some((len_str, rest_str)) = encoded_value.split_once(':') {
            let len = len_str.parse::<usize>().expect("valid unsigned digit");
            // eprintln!("Inside DIGIT");
            // eprintln!("rest_str is {rest_str}");
            // eprintln!("len is {len}");
            let word = &rest_str[..len];
            // eprintln!("word is {word}");
            let rest = &rest_str[len..];
            // eprintln!("rest is {rest}");
            return Ok((serde_json::Value::String(word.to_string()), rest));
        }
        return Err("There is no : in the encoded value");
    } else if encoded_value.starts_with("i") {
        if let Some((prefix_str, suffix)) = encoded_value.split_once('e') {
            let prefix = &prefix_str[1..];
            // eprintln!("Inside I");
            // eprintln!("prefix is {prefix}");
            // eprintln!("suffix is {suffix}");
            let number = prefix.parse::<i64>().expect("valid int64 number");
            // eprintln!("Number is {number}");
            return Ok((serde_json::Value::Number(number.into()), suffix));
        }
        return Err("There is no 'e' in the encoded value");
    } else if encoded_value.starts_with("l") {
        let mut ans = Vec::new();
        let mut rest = &encoded_value[1..];
        // eprintln!("rest is {rest}");
        while !rest.is_empty() && !rest.starts_with('e') {
            let (result, rest_str) = decode_bencoded_value(rest).ok().expect("String should be valid");
            // eprintln!("result is {result}, remainder is {rest_str}");
            ans.push(result);
            rest = rest_str;
        }

        return Ok((serde_json::Value::Array(ans), &rest[1..]));
    } else if encoded_value.starts_with("d") {
        let mut ans = serde_json::Map::new();
        let mut rest = &encoded_value[1..];
        while !rest.is_empty() && !rest.starts_with('e') {
            let (result, rest_str) = decode_bencoded_value(rest).ok().expect("String should be valid");
            rest = rest_str;
            let key = result.as_str().expect("Key should be a string");
            // eprintln!("key is {key}");
            let (value, rest_str) = decode_bencoded_value(rest).ok().expect("String should be valid");
            rest = rest_str;
            ans.insert(String::from(key), value);
        }

        return Ok((serde_json::Value::Object(ans), &rest[1..]));
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
        let (decoded_value, _) = decode_bencoded_value(encoded_value).ok().expect("Should only panic and not return an error");
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}

// lli543e9:blueberryee
// lli4eei5ee
