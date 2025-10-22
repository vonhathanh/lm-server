use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

// use &str instead of &String because String is already a container for a str that is stored on the heap.
// &String is a pointer to poiner, this is unnecessary
pub trait IRequest {
    fn get_method(&self) -> &str;
    fn get_path(&self) -> &str;
    fn get_query(&self) -> &[(String, String)];
}

pub trait IResponse {
    fn len(&self) -> usize;
    fn body(&self) -> &str;
}

#[derive(Debug)]
// for data-owning struct: use String is a better choice. We don't have to deal with lifetime and can modify data
// at our own will. If we want &str we can expose methods that return &str
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: String,
}
pub struct Response(pub String);

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
}

impl IResponse for Response {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn body(&self) -> &str {
        &self.0
    }
}

type RequestHandler = Box<dyn Fn(&dyn IRequest) -> Box<dyn IResponse>>;

pub struct Server {
    // server own route data. It want to store them permanently until shutdown,
    // not borrow them temporarily
    routes: HashMap<String, HashMap<String, Route>>,
}

struct Route {
    handler: RequestHandler,
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
        let route = Route::new(Box::new(handler));
        root.insert(path.to_string(), route);
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("POST".to_string())
            .or_insert(HashMap::new());
        let route = Route::new(Box::new(handler));
        root.insert(path.to_string(), route);
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("PUT".to_string())
            .or_insert(HashMap::new());
        let route = Route::new(Box::new(handler));
        root.insert(path.to_string(), route);
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self
            .routes
            .entry("DELETE".to_string())
            .or_insert(HashMap::new());
        let route = Route::new(Box::new(handler));
        root.insert(path.to_string(), route);
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        let content: Vec<String> = buf_reader
            .lines()
            .map(|value| value.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request content: {content:#?}");
        let request: Request = parse_request(&content[0]);

        self.match_request(&request, &mut stream);
    }

    fn match_request(&self, request: &impl IRequest, stream: &mut TcpStream) {
        let method = request.get_method();
        let root = self.routes.get(&method).unwrap();
        let handler = root.get(request.get_path()).unwrap();
        let response = handler.handler.as_ref()(request);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            response.len(),
            response.body()
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}

impl Route {
    fn new(handler: RequestHandler) -> Self {
        Route { handler }
    }
}
