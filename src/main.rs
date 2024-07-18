use std::{
    collections::HashMap,
    env,
    fs::read,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    str::FromStr,
    thread,
};

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
            let request_path = request_split[1];
            if request_path == "/" {
                stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("error");
                return;
            }
            let paths: Vec<&str> = request_path
                .split("/")
                .collect::<Vec<&str>>()
                .into_iter()
                .filter(|v| *v != "")
                .collect();
            let main_path = paths[0];
            match main_path {
                "echo" => {
                    let value = paths[1];
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
                "files" => {
                    let args: Vec<String> = env::args().collect();
                    let is_directory = &args[1] == "--directory";
                    if is_directory {
                        let file_name = paths[1];
                        let mut file_path = PathBuf::from_str(&args[2]).expect("Invalid argument");
                        file_path.push(file_name);
                        let content = read(file_path).expect("No Content");
                        let content_str = String::from_utf8(content).expect("Failed parsing utf8");
                        stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", content_str.len(), content_str).as_bytes()).expect("error");
                    }
                    return;
                }
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
