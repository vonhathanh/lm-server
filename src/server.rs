use std::{
    collections::HashMap, fmt::{format, write, Display}, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub trait IRequest {}
pub trait IResponse {}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path_variables: Vec<String>,
    pub query_parameters: Vec<(String, String)>,
    pub body: String,
}
pub struct Response(pub String);

impl IRequest for Request {}

impl IResponse for Response {}

type RequestHandler<T, V> = fn(value: T) -> V;

pub struct Server<T, V>
where
    T: IRequest,
    V: IResponse,
{
    get_routes: Route<T, V>,
    post_routes: Route<T, V>,
    delete_routes: Route<T, V>,
    put_routes: Route<T, V>,
}

#[derive(Debug)]
struct Route<T, V> {
    path: String,
    placeholder: Option<String>,
    handler: Option<RequestHandler<T, V>>,
    sub_routes: HashMap<String, Route<T, V>>,
}

impl<T: IRequest, V: IResponse> Server<T, V> {
    pub fn new() -> Self {
        Server {
            get_routes: Route {path: "/".to_string(), placeholder: None, handler: None, sub_routes: HashMap::new() },
            post_routes: Route {path: "/".to_string(), placeholder: None, handler: None, sub_routes: HashMap::new() },
            put_routes: Route {path: "/".to_string(), placeholder: None, handler: None, sub_routes: HashMap::new() },
            delete_routes: Route {path: "/".to_string(), placeholder: None, handler: None, sub_routes: HashMap::new() },
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        println!("GETs: {}", self.get_routes);
        println!("POSTs: {}", self.post_routes);
        println!("PUTs: {}", self.put_routes);
        println!("DELETEs: {}", self.delete_routes);
        println!("Server is listening at: {SERVER_ADDRESS}");
        for stream in listener.incoming() {
            let tcp_stream = stream.unwrap();
            self.handle_connection(tcp_stream);
        }
    }

    pub fn get(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.get_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn post(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.post_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn put(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.put_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn delete(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.delete_routes.add(path.split_terminator('/').collect(), handler);
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        // let request_info: RequestInfo = RequestInfo::new(first_line);

        let content: Vec<String> = buf_reader
            .lines()
            .map(|value| value.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request content: {content:#?}");
        let request: Request = parse_request(&content[0]);

        self.match_request(&request);

        // if content[0] == "GET / HTTP/1.1" {
        //     let response = format!(
        //         "{status_line}\r\nContent-Length: {}\r\n\r\n{response}",
        //         response.len()
        //     );
        //     stream.write_all(response.as_bytes()).unwrap();
        // } else {
        //     let response = format!(
        //         "{status_line}\r\nContent-Length: {}\r\n\r\n{response}",
        //         response.len()
        //     );
        //     stream.write_all(response.as_bytes()).unwrap();
        // }
    }

    fn match_request(&self, request: &Request) {
        if request.method == "GET" {
            self.get_routes.handle(request);
        } else if request.method == "POST" {
            self.post_routes.handle(request);
        } else if request.method == "PUT" {
            self.put_routes.handle(request);
        } else if request.method == "DELETE" {
            self.delete_routes.handle(request);
        } else {
            panic!("Unrecognize request method!")
        }
    }
}

impl<T, V> Route<T, V> {
    fn new(
        path: String,
        placeholder: Option<String>,
        handler: Option<RequestHandler<T, V>>,
        sub_routes: HashMap<String, Route<T, V>>,
    ) -> Self {
        Route {
            path,
            placeholder,
            handler,
            sub_routes,
        }
    }

    fn add(&mut self, mut tokens: Vec<&str>, handler: RequestHandler<T, V>) {
        if tokens.len() == 0 {
            return;
        }

        if tokens[0] == "".to_string() && tokens.len() > 1 {
            tokens = tokens.split_off(1);
        }

        // skip the first token if it's the index root
        let root = tokens[0];
        let mut placeholder: Option<String> = None;

        // index root (/) detected, just need to update handler function
        if self.path == "/".to_string() && tokens.len() == 1 {
            self.handler = Some(handler);
            return;
        }

        // placeholder detection likes :id, :name...
        if root.find(':') == Some(0) {
            placeholder = Some(root[1..].to_string());
        }

        // this route is already added
        if self.sub_routes.contains_key(root) {
            // not the end of the full route, delegate the add route task to it's corresponding sub route.
            if tokens.len() > 1 {
                let route = self.sub_routes.get_mut(root).unwrap();
                route.add(tokens.split_off(1), handler);
            }
        } else {
            let path = format!("{}{}/", self.path, root);
            let mut route;
            if tokens.len() == 1 {
                route = Route::new(path, placeholder, Some(handler), HashMap::new());
            } else {
                route = Route::new(path, placeholder, None, HashMap::new());
            }
            route.add(tokens.split_off(1), handler);
            self.sub_routes.entry(root.to_string()).or_insert(route);
        }
    }

    fn handle(&self, request: &Request) {}
}

impl<T, V> Display for Route<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sub_routes.len() == 0{
            write!(f, "{}\n", self.path).unwrap();
        } 

        for (_, route) in &self.sub_routes {
            route.fmt(f).unwrap();
        }
        
        Ok(())
    }
}