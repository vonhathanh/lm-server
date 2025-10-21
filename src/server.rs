use std::{
    collections::HashMap, io::{BufRead, BufReader}, net::{TcpListener, TcpStream}
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub trait IRequest {
    fn get_method(&self) -> String;
    fn get_path(&self) -> &String;
    fn get_query(&self) -> &Vec<(String, String)>;
}

pub trait IResponse {
    fn len(&self) -> usize;
    fn body(&self) -> &String;
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: String,
}
pub struct Response(pub String);

impl IRequest for Request {
    fn get_method(&self) -> String {
        self.method.clone()
    }

    fn get_path(&self) -> &String {
        &self.path
    }

    fn get_query(&self) -> &Vec<(String, String)> {
        &self.query
    }
}

impl IResponse for Response {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn body(&self) -> &String {
        &self.0
    }
}

type RequestHandler = Box<dyn Fn(&dyn IRequest) -> Box<dyn IResponse>>;

pub struct Server {
    routes: HashMap<String, HashMap<String, Route>>
}

struct Route {
    handler: RequestHandler,
}

impl Server {
    pub fn new() -> Self {
        Server {
            routes: HashMap::new()
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
        // println!("GETs: {}", self.get_routes);
        // println!("POSTs: {}", self.post_routes);
        // println!("PUTs: {}", self.put_routes);
        // println!("DELETEs: {}", self.delete_routes);
        println!("Server is listening at: {SERVER_ADDRESS}");
        for stream in listener.incoming() {
            let tcp_stream = stream.unwrap();
            self.handle_connection(tcp_stream);
        }
    }

    pub fn get<F>(&mut self, path: String, handler: F) 
    where F: Fn(&dyn IRequest) -> Box<dyn IResponse> + 'static,
    {
        let root = self.routes.entry("GET".to_string()).or_insert(HashMap::new());
        let route = Route::new(Box::new(handler));
        root.insert(path, route);
    }

    // pub fn post(&mut self, path: String, handler: RequestHandler) {
    //     self.post_routes.add(path.split_terminator('/').collect(), handler);
    // }

    // pub fn put(&mut self, path: String, handler: RequestHandler) {
    //     self.put_routes.add(path.split_terminator('/').collect(), handler);
    // }

    // pub fn delete(&mut self, path: String, handler: RequestHandler) {
    //     self.delete_routes.add(path.split_terminator('/').collect(), handler);
    // }

    fn handle_connection(&self, mut stream: TcpStream) -> Box<dyn IResponse> {
        let buf_reader = BufReader::new(&stream);
        let content: Vec<String> = buf_reader
            .lines()
            .map(|value| value.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request content: {content:#?}");
        let request: Request = parse_request(&content[0]);

        self.match_request(&request)

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

    fn match_request(&self, request: &impl IRequest) -> Box<dyn IResponse> {
        let method = request.get_method();
        let root = self.routes.get(&method).unwrap();
        let handler = root.get(request.get_path()).unwrap();
        handler.handler.as_ref()(request)
    }
}

impl Route 
{
    fn new(
        handler: RequestHandler,
    ) -> Self {
        Route {
            handler,
        }
    }
}
