use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                let res = _stream.read(&mut buffer[..]);
                match res {
                    Ok(a) => {
                        let x = &buffer[..a];
                        let msg = String::from_utf8(x.to_vec()).expect("error");
                        let msgs: Vec<&str> = msg.split(" ").collect();
                        if msgs[1] == "/" {
                            _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("error");
                            continue;
                        }
                        let paths: Vec<&str> = msgs[1]
                            .split("/")
                            .collect::<Vec<&str>>()
                            .into_iter()
                            .filter(|v| *v != "")
                            .collect();
                        if paths[0] == "echo" {
                            let value = paths[1];
                            _stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\n{}\r\n\r\n", value).as_bytes()).expect("error");
                            println!("{:?}", paths);
                            continue;
                        }
                        _stream
                            .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                            .expect("error");
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        };
    }
}
