use core::str;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn parse(data: &str) -> HttpRequest {
        let splited = data.split("\r\n\r\n").collect::<Vec<&str>>();
        let upper = splited[0].split("\r\n").collect::<Vec<&str>>();

        let mut request_lines = upper[0].split_whitespace();

        let headers =
            upper[1..].iter().fold(HashMap::new(), |mut headers, line| {
                let mut splited = line.split(":");
                headers.insert(
                    String::from(splited.next().unwrap().trim()),
                    String::from(splited.next().unwrap().trim()),
                );
                headers
            });
        HttpRequest {
            method: String::from(request_lines.next().unwrap()),
            path: String::from(request_lines.next().unwrap()),
            headers,
            body: Vec::from(splited[0].as_bytes()),
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let re = Regex::new(r"/echo/(?<echo_str>.+)").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut bufs = [0u8; 512];
                _ = match stream.read(&mut bufs) {
                    Ok(_) => match str::from_utf8(&bufs) {
                        Ok(data) => {
                            let req = HttpRequest::parse(data);

                            if req.path == "/" {
                                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                            } else if re.is_match(&req.path) {
                                let echo = re
                                    .captures(&req.path)
                                    .unwrap()
                                    .name("echo_str")
                                    .unwrap()
                                    .as_str();
                                stream.write_all(
                                    format!(
                                        "\
                                HTTP/1.1 200 OK\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: {}\r\n\
                                \r\n{}",
                                        echo.len(),
                                        echo,
                                    )
                                    .as_bytes(),
                                )
                            } else if req.path == "/user-agent" {
                                let user_agent =
                                    req.headers.get("User-Agent").unwrap();

                                stream.write_all(
                                    format!(
                                        "\
                                    HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: {}\r\n\
                                    \r\n\
                                    {}",
                                        user_agent.len(),
                                        user_agent
                                    )
                                    .as_bytes(),
                                )
                            } else {
                                stream.write_all(
                                    b"HTTP/1.1 404 Not Found\r\n\r\n",
                                )
                            }
                        }
                        Err(error) => {
                            println!("could not read: {error:?}");
                            stream
                                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                        }
                    },
                    Err(error) => {
                        println!("could not read: {error:?}");
                        stream.write_all(
                            b"HTTP/1.1 500 Internal Server Error\r\n\r\n",
                        )
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
