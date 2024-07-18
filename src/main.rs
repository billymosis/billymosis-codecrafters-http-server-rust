use std::{
    collections::HashMap,
    env,
    fs::{self, read},
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    str::FromStr,
    thread,
};

use flate2::{write::ZlibEncoder, Compression};

fn handle_client(mut stream: std::net::TcpStream) {
    println!("accepted new connection");
    let mut buffer = [0; 1024];
    let res = stream.read(&mut buffer[..]);
    match res {
        Ok(a) => {
            let x = &buffer[..a];
            let msg = String::from_utf8(x.to_vec()).expect("error");
            let msgs: Vec<&str> = msg.split("\r\n").collect();
            let (request_lines, headers) = msgs.split_at(1);
            let request_split: Vec<&str> = request_lines[0].split(" ").collect();
            let mut header_hash = HashMap::new();
            for x in headers {
                let split: Vec<&str> = x.split(": ").collect();
                if split.len() == 2 {
                    header_hash.insert(split[0], split[1]);
                }
            }
            let method = request_split[0];
            let request_path = request_split[1];
            let paths: Vec<&str> = request_path
                .split("/")
                .collect::<Vec<&str>>()
                .into_iter()
                .filter(|v| *v != "")
                .collect();
            let main_path = paths.get(0).unwrap_or(&"/").to_owned();
            match main_path {
                "/" => {
                    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("error");
                    return;
                }
                "echo" => {
                    let value = paths[1];
                    if header_hash
                        .get("Accept-Encoding")
                        .unwrap_or(&"")
                        .contains("gzip")
                    {
                        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                        let _ = e.write_all(value.as_bytes());
                        let compressed_bytes = e.finish().unwrap();
                        let len = compressed_bytes.len();
                        let b = b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: text/plain\r\n";
                        let c = format!("Content-Length: {}\r\n\r\n", len)
                            .as_bytes()
                            .to_owned();
                        let mut res = Vec::new();
                        res.extend_from_slice(b);
                        res.extend_from_slice(&c);
                        res.extend_from_slice(&compressed_bytes);
                        stream.write(&res).expect("error");
                        return;
                    }
                    stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n", value.len(), value).as_bytes()).expect("error");
                    return;
                }
                "user-agent" => {
                    let op = header_hash.get("User-Agent");
                    match op {
                        Some(value) => {
                            stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n", value.len(), value).as_bytes()).expect("error");
                        }
                        None => return,
                    }
                }
                "files" => match method {
                    "GET" => {
                        let args: Vec<String> = env::args().collect();
                        let is_directory = &args[1] == "--directory";
                        if is_directory {
                            let file_name = paths[1];
                            let mut file_path =
                                PathBuf::from_str(&args[2]).expect("Invalid argument");
                            file_path.push(file_name);
                            let content = read(file_path);
                            match content {
                                Ok(content_str) => {
                                    let str = String::from_utf8(content_str)
                                        .expect("Failed parsing utf8");
                                    stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", str.len(), str).as_bytes()).expect("error");
                                }
                                Err(_) => {
                                    stream
                                        .write(format!("HTTP/1.1 404 Not Found\r\n\r\n").as_bytes())
                                        .expect("error");
                                }
                            }
                        }
                        return;
                    }
                    "POST" => {
                        let args: Vec<String> = env::args().collect();
                        let is_directory = &args[1] == "--directory";
                        if is_directory {
                            let file_name = paths[1];
                            let mut file_path =
                                PathBuf::from_str(&args[2]).expect("Invalid argument");
                            file_path.push(file_name);
                            let body = msgs.last().expect("No body");
                            let result = fs::write(file_path, body);
                            match result {
                                Ok(_) => {
                                    stream
                                        .write(format!("HTTP/1.1 201 Created\r\n\r\n").as_bytes())
                                        .expect("error");
                                }
                                Err(_) => {
                                    stream
                                        .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                        .expect("error");
                                }
                            }
                        }
                        return;
                    }
                    _ => {
                        stream
                            .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                            .expect("error");
                    }
                },
                _ => {
                    stream
                        .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        .expect("error");
                }
            }
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
}
