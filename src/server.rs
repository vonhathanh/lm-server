use std::{
    collections::HashMap,
    fmt,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub enum HttpStatus {
    OK,
    BAD_REQUEST,
    NOT_FOUND,
    UNAUTHENTICATED,
    UNAUTHORIZED,
    INTERNAL_SERVER_ERROR,
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpStatus::OK => write!(f, "HTTP/1.1 200 OK"),
            HttpStatus::BAD_REQUEST => write!(f, "HTTP/1.1 400 Bad Request"),
            HttpStatus::NOT_FOUND => write!(f, "HTTP/1.1 404 Not Found"),
            HttpStatus::UNAUTHENTICATED => write!(f, "HTTP/1.1 401 Unauthorized"),
            HttpStatus::UNAUTHORIZED => write!(f, "HTTP/1.1 403 Forbidden"),
            HttpStatus::INTERNAL_SERVER_ERROR => write!(f, "HTTP/1.1 500 Internal Server Error"),
        }
    }
}

// use &str instead of &String because String is already a container for a str that is stored on the heap.
// &String is a pointer to poiner, this is unnecessary
pub trait IRequest {
    fn get_method(&self) -> &str;
    fn get_path(&self) -> &str;
    fn get_query(&self) -> &[(String, String)];
    fn get_body(&self) -> &str;
}

pub trait IResponse {
    fn len(&self) -> usize;
    fn body(&self) -> &str;
    fn status(&self) -> &str {
        "HTTP/1.1 200 OK"
    }
}

#[derive(Debug)]
// for data-owning struct: use String is a better choice. We don't have to deal with lifetime and can modify data
// at our own will. If we want &str we can expose methods that return &str
pub struct Request {
    method: String,
    path: String,
    query: Vec<(String, String)>,
    body: String,
}
pub struct Response {
    body: String,
    status: String,
}

impl Request {
    pub fn new(method: String, path: String, query: Vec<(String, String)>, body: String) -> Self {
        Request {
            method,
            path,
            query,
            body,
        }
    }
}

impl IRequest for Request {
    fn get_method(&self) -> &str {
        &self.method
    }

    fn get_path(&self) -> &str {
        &self.path
    }

    fn get_query(&self) -> &[(String, String)] {
        &self.query
    }

    fn get_body(&self) -> &str {
        &self.body
    }
}

impl IResponse for Response {
    fn len(&self) -> usize {
        self.body.len()
    }

    fn body(&self) -> &str {
        &self.body
    }

    fn status(&self) -> &str {
        &self.status
    }
}

impl Response {
    pub fn new(body: String, status: String) -> Self {
        Response {
            body: body,
            status: status,
        }
    }
}

// a Boxed closure/function that accepts a reference to any object that implements IRequest trait and
// return a boxed object that implement IResponse trait
// both must be Boxed because trait objec (DST) can't be stored or returned directly without indirection
pub type RequestHandler = Box<dyn Fn(&dyn IRequest) -> Box<dyn IResponse>>;

pub struct Server {
    // server own route data. It want to store them permanently until shutdown,
    // not borrow them temporarily
    routes: HashMap<String, HashMap<String, RequestHandler>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            routes: HashMap::new(),
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        println!("Server is listening at: {SERVER_ADDRESS}");
        for stream in listener.incoming() {
            let tcp_stream = stream.unwrap();
            self.handle_connection(tcp_stream);
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("GET".to_string())
            .or_insert(HashMap::new());
        root.insert(path.to_ascii_lowercase(), Box::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("POST".to_string())
            .or_insert(HashMap::new());
        root.insert(path.to_ascii_lowercase(), Box::new(handler));
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("PUT".to_string())
            .or_insert(HashMap::new());
        root.insert(path.to_ascii_lowercase(), Box::new(handler));
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("DELETE".to_string())
            .or_insert(HashMap::new());
        root.insert(path.to_ascii_lowercase(), Box::new(handler));
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buf_reader = BufReader::new(&stream);
        // read request line
        let mut request_line: String = String::new();
        buf_reader.read_line(&mut request_line).unwrap();
        let mut request: Request = parse_request(&request_line);

        // read header
        let mut header = HashMap::new();
        loop {
            let mut buf = String::new();

            buf_reader.read_line(&mut buf).unwrap();

            if buf.trim().is_empty() {
                break;
            }

            if let Some((key, value)) = buf.split_once(':') {
                header.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // read body
        let body = if let Some(str_len) = header.get("content-length") {
            let content_length = str_len.parse().unwrap_or(0);
            let mut buffer = vec![0u8; content_length];
            buf_reader.read_exact(&mut buffer).unwrap();
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            String::new()
        };

        request.body = body;

        self.match_request(&request, &mut stream);
    }

    fn match_request(&self, request: &impl IRequest, stream: &mut TcpStream) {
        let method = request.get_method();
        let root = self.routes.get(method).unwrap();
        let handler = root.get(request.get_path()).unwrap();
        let response = handler.as_ref()(request);
        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            response.status(),
            response.len(),
            response.body()
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
