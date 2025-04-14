use anyhow::Error;
use core::str;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn parse(data: &str) -> Option<HttpRequest> {
        let splited = data.split("\r\n\r\n").collect::<Vec<&str>>();
        let upper = splited[0].split("\r\n").collect::<Vec<&str>>();

        let request_lines: Vec<&str> = upper[0].split_whitespace().collect();

        if request_lines.len() < 2 || !request_lines[2].starts_with("HTTP") {
            return None;
        }

        let headers =
            upper[1..].iter().fold(HashMap::new(), |mut headers, line| {
                let mut splited = line.split(":");
                headers.insert(
                    String::from(splited.next().unwrap().trim()),
                    String::from(splited.next().unwrap().trim()),
                );
                headers
            });
        Some(HttpRequest {
            method: String::from(request_lines[0]),
            path: String::from(request_lines[1]),
            headers,
            body: Vec::from(splited[0].as_bytes()),
        })
    }
}

fn handle(stream: &mut TcpStream) -> (String, Option<String>) {
    let mut buf = [0u8; 512];
    if let Err(err) = stream.read(&mut buf) {
        return (
            String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n"),
            Some(format!("could not read: {err:?}")),
        );
    }

    let data = match str::from_utf8(&buf) {
        Ok(data) => data,
        Err(err) => {
            return (
                String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
                Some(format!("could not create a buf string: {err:?}")),
            );
        }
    };

    let req = match HttpRequest::parse(data) {
        Some(req) => req,
        None => {
            return (String::from("HTTP/1.1 200 OK\r\n"), None);
        }
    };

    let re = Regex::new(r"/echo/(?<echo_str>.+)").unwrap();

    if req.path == "/" {
        (String::from("HTTP/1.1 200 OK\r\n\r\n"), None)
    } else if re.is_match(&req.path) {
        let echo = re
            .captures(&req.path)
            .unwrap()
            .name("echo_str")
            .unwrap()
            .as_str();
        (
            format!(
                "\
                                HTTP/1.1 200 OK\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: {}\r\n\
                                \r\n{}",
                echo.len(),
                echo,
            ),
            None,
        )
    } else if req.path == "/user-agent" {
        let user_agent = req.headers.get("User-Agent").unwrap();

        (
            format!(
                "\
                                    HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: {}\r\n\
                                    \r\n\
                                    {}",
                user_agent.len(),
                user_agent
            ),
            None,
        )
    } else {
        (String::from("HTTP/1.1 404 Not Found\r\n\r\n"), None)
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        println!("accpeting a connection...");
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("an error occurred: {err:?}");
                continue;
            }
        };

        std::thread::spawn(move || {
            let (response, error) = handle(&mut stream);
            if let Some(error) = error {
                eprintln!("{error}");
            }
            _ = stream.write_all(response.as_bytes());
        });
    }
}
