mod http_request;

use core::str;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::{env, fs};

use http_request::HttpRequest;

fn handle(
    stream: &mut TcpStream,
    dir: Option<String>,
) -> (String, Option<String>) {
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
        None => return (String::from("HTTP/1.1 200 OK\r\n"), None),
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
    } else if req.path.starts_with("/files/") {
        let filename = req.path.trim_start_matches("/files/");
        let dir = dir.unwrap_or(String::from("/tmp/"));
        let filename = &format!("{dir}{filename}");

        if req.method == "POST" {
            println!("creating a file: {filename}");
            match req.body {
                None => (
                    String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
                    Some(String::from("no payload found")),
                ),
                Some(mut content) => {
                    content = content.replace('\x00', "");
                    if let Err(err) = fs::write(filename, content) {
                        (
                            String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
                            Some(format!("could not write file: {err:?}")),
                        )
                    } else {
                        (String::from("HTTP/1.1 201 Created\r\n\r\n"), None)
                    }
                }
            }
        } else {
            println!("reading a file: {filename}",);
            match fs::read(filename) {
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        (
                            String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
                            Some(format!(
                                "no file found, filename: {filename}"
                            )),
                        )
                    } else {
                        (
                            String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
                            Some(format!("could not read a file: {err:?}")),
                        )
                    }
                }
                Ok(content) => {
                    let content =
                        str::from_utf8(&content).unwrap().replace("\x00", "");
                    (
                        format!(
                            "\
                    HTTP/1.1 200 OK\r\n\
                    Content-Type: application/octet-stream\r\n\
                    Content-Length: {}\r\n\
                    \r\n\
                    {}
                    ",
                            content.len(),
                            content,
                        ),
                        None,
                    )
                }
            }
        }
    } else {
        (String::from("HTTP/1.1 404 Not Found\r\n\r\n"), None)
    }
}

const ARGS: [&str; 1] = ["--directory"];

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!!!");
    // Uncomment this block to pass the first stage

    let args_set = HashSet::from(ARGS);
    let args: Vec<String> = env::args().collect();
    let mut passed_args: HashMap<String, String> = HashMap::new();
    for (i, arg) in args[1..].iter().enumerate() {
        // TODO: args_set to HashMap to map with its properties such as, is option value need?
        if i % 2 == 1 {
            continue;
        }
        if !args_set.contains(arg.as_str()) {
            eprintln!("unknown an option: {arg}");
            exit(-1);
        }
        let i = args.iter().position(|v| v == arg).unwrap();
        let value = &args[i + 1];
        passed_args.insert(arg.clone(), value.clone());
    }

    let dir_key = &String::from(ARGS[0]);
    let mut dir: Option<String> = None;
    if passed_args.contains_key(dir_key) {
        let arg_dir = &passed_args.get(dir_key).unwrap().clone();
        println!("creating a directory: {arg_dir}");
        if let Err(err) = fs::create_dir_all(arg_dir) {
            eprintln!(
                "could not create a directory: dir: {arg_dir}, error: {err}"
            );
            exit(-1);
        }
        dir = Some(arg_dir.clone());
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        println!("accpeting a new connection...");
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("an error occurred: {err:?}");
                continue;
            }
        };

        let dir = dir.clone();
        std::thread::spawn(move || {
            let (response, error) = handle(&mut stream, dir);
            if let Some(error) = error {
                eprintln!("{error}");
            }
            _ = stream.write_all(response.as_bytes());
        });
    }
}
