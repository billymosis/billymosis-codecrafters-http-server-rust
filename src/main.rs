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
                let mut buffer = [0; 10];
                let res = _stream.read(&mut buffer[..]);
                match res {
                    Ok(a) => {
                        let x = &buffer[..a];
                        let msg = String::from_utf8(x.to_vec()).expect("error");
                        let msgs: Vec<&str> = msg.split(" ").collect();
                        print!("{:?}", msg);
                        if msgs[1] == "/" {
                            _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("error");
                        } else {
                            _stream
                                .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                .expect("error");
                        }
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
