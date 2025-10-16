use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub trait IRequest {}
pub trait IResponse {}

pub struct Request {
    pub method: String,
    pub path_variables: Vec<String>,
    pub query_parameters: Vec<(String, String)>,
    pub body: String
}
pub struct Response(String);

impl IRequest for Request {}

impl IResponse for Response {}

type RequestHandler<T, V> = fn(value: T) -> V;

pub struct Server<T, V>
where
    T: IRequest,
    V: IResponse,
{
    get_routes: RouteCollection<T, V>,
    post_routes: RouteCollection<T, V>,
    delete_routes: RouteCollection<T, V>,
    put_routes: RouteCollection<T, V>,
}

struct RouteCollection<T, V> {
    routes: HashMap<String, Route<T, V>>,
}

struct Route<T, V> {
    placeholder: Option<String>,
    handler: Option<RequestHandler<T, V>>,
    sub_routes: HashMap<String, Route<T, V>>,
}

impl<T: IRequest, V: IResponse> Server<T, V> {
    pub fn new() -> Self {
        Server {
            get_routes: RouteCollection::new(),
            post_routes: RouteCollection::new(),
            put_routes: RouteCollection::new(),
            delete_routes: RouteCollection::new(),
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

    pub fn get(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.get_routes.add(path, handler);
    }

    pub fn post(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.post_routes.add(path, handler);
    }

    pub fn put(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.put_routes.add(path, handler);
    }

    pub fn delete(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.delete_routes.add(path, handler);
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

    fn match_request(&self, request: &Request){
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

impl<T, V> RouteCollection<T, V> {
    fn new() -> Self {
        RouteCollection {
            routes: HashMap::new(),
        }
    }

    fn add(&mut self, path: String, handler: RequestHandler<T, V>) {
        let mut tokens: Vec<&str> = path.split_terminator('/').collect();
        let root = tokens[0];
        let mut placeholder: Option<String> = None;

        if root.find(':') == Some(1) {
            placeholder = Some(root[1..].to_string());
        }
        if self.routes.contains_key(root) {
            let route = self.routes.get_mut(root).unwrap();
            if tokens.len() > 1 {
                route.add(tokens.split_off(1), handler);
            }
        } else {
            let mut route ;
            if tokens.len() == 1 {
                route = Route::new(placeholder, Some(handler), HashMap::new());
            } else {
                route = Route::new(placeholder, None, HashMap::new());
            }
            route.add(tokens.split_off(1), handler);
            self.routes.entry(root.to_string()).or_insert(route);
        }
    }

    fn handle(&self, request: &Request) {

    }
}

impl<T, V> Route<T, V> {
    fn new(
        placeholder: Option<String>,
        handler: Option<RequestHandler<T, V>>,
        sub_routes: HashMap<String, Route<T, V>>,
    ) -> Self {
        Route {
            placeholder,
            handler,
            sub_routes,
        }
    }

    fn add(&mut self, mut tokens: Vec<&str>, handler: RequestHandler<T, V>) {
        if tokens.len() == 0 {
            return;
        }

        let root = tokens[0];
        let mut placeholder: Option<String> = None;

        if root.find(':') == Some(0) {
            placeholder = Some(root[1..].to_string());
        }

        if self.sub_routes.contains_key(root) {
            let route = self.sub_routes.get_mut(root).unwrap();
            if tokens.len() > 1 {
                route.add(tokens.split_off(1), handler);
            }
        } else {
            let mut route ;
            if tokens.len() == 1 {
                route = Route::new(placeholder, Some(handler), HashMap::new());
            } else {
                route = Route::new(placeholder, None, HashMap::new());
            }
            route.add(tokens.split_off(1), handler);
            self.sub_routes.entry(root.to_string()).or_insert(route);
        }
    }
}
