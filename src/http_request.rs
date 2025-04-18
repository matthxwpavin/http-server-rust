use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn parse(data: &str) -> Option<HttpRequest> {
        let splited = data.split("\r\n\r\n").collect::<Vec<&str>>();
        let upper = splited[0].split("\r\n").collect::<Vec<&str>>();

        let request_lines: Vec<&str> = upper[0].split_whitespace().collect();

        if request_lines.len() < 2 || !request_lines[2].starts_with("HTTP") {
            return None;
        }

        let headers =
            upper[1..].iter().fold(HashMap::new(), |mut headers, line| {
                let mut splited = line.split(":");
                let key = splited.next().unwrap().trim();
                let value = splited.next().unwrap().trim();
                let values = if value.contains(", ") {
                    value.split(", ").map(String::from).collect::<Vec<String>>()
                } else {
                    vec![String::from(value)]
                };
                headers.insert(String::from(key), values);
                headers
            });
        Some(HttpRequest {
            method: String::from(request_lines[0]),
            path: String::from(request_lines[1]),
            headers,
            body: splited.get(1).map(|body| String::from(*body)),
        })
    }
}
