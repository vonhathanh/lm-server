use std::{
    collections::HashMap, fmt::{Display}, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}
};

use crate::server_utils::parse_request;

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub trait IRequest {
    fn get_method(&self) -> String;
    fn get_path_variables(&self) -> &Vec<String>;
    fn get_query_parameters(&self) -> &Vec<(String, String)>;
}
pub trait IResponse {}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path_variables: Vec<String>,
    pub query_parameters: Vec<(String, String)>,
    pub body: String,
}
pub struct Response(pub String);

impl IRequest for Request {
    fn get_method(&self) -> String {
        self.method.clone()
    }

    fn get_path_variables(&self) -> &Vec<String> {
        &self.path_variables
    }

    fn get_query_parameters(&self) -> &Vec<(String, String)> {
        &self.query_parameters
    }
}

impl IResponse for Response {}

type RequestHandler = Box<dyn Fn(&dyn IRequest) -> Box<dyn IResponse> + Send + Sync>;

pub struct Server
{
    get_routes: Route,
    post_routes: Route,
    delete_routes: Route,
    put_routes: Route,
}

struct Route {
    path: String,
    placeholder: Option<String>,
    handler: Option<RequestHandler>,
    sub_routes: HashMap<String, Route>,
}

impl Server {
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

    pub fn get<F>(&mut self, path: String, handler: F) 
    where F: Fn(&dyn IRequest) -> Box<dyn IResponse> + Send + Sync + 'static,
    {
        self.get_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn post(&mut self, path: String, handler: RequestHandler) {
        self.post_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn put(&mut self, path: String, handler: RequestHandler) {
        self.put_routes.add(path.split_terminator('/').collect(), handler);
    }

    pub fn delete(&mut self, path: String, handler: RequestHandler) {
        self.delete_routes.add(path.split_terminator('/').collect(), handler);
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

    fn match_request(&self, request: &impl IRequest) {
        let method = request.get_method();
        if method == "GET" {
            self.get_routes.handle(request);
        } else if method == "POST" {
            self.post_routes.handle(request);
        } else if method == "PUT" {
            self.put_routes.handle(request);
        } else if method == "DELETE" {
            self.delete_routes.handle(request);
        } else {
            panic!("Unrecognize request method!")
        }
    }
}

impl Route 
{
    fn new(
        path: String,
        placeholder: Option<String>,
        handler: Option<RequestHandler>,
        sub_routes: HashMap<String, Route>,
    ) -> Self {
        Route {
            path,
            placeholder,
            handler,
            sub_routes,
        }
    }

    fn prepare_input(&mut self, tokens: &mut Vec<&str>) -> (String, Option<String>) {
        // skip the first token if it's the index root
        if tokens[0] == "".to_string() && tokens.len() > 1 {
            // TODO: use devec to pop the index
            tokens.remove(0);
        }

        let root = tokens[0];

        let placeholder = if root.find(':') == Some(0) {
            Some(root[1..].to_string())
        } else {
            None
        };

        return (root.to_owned(), placeholder)
    }

    fn add<F>(&mut self, mut tokens: Vec<&str>, handler: F) 
    where F: Fn(&dyn IRequest) -> Box<dyn IResponse> + Send + Sync + 'static,
    {
        if tokens.len() == 0 {
            return;
        }
        // index Route (/) is detected, just need to update handler function
        if self.path == "/".to_string() && tokens.len() == 1 {
            self.handler = Some(Box::new(handler));
            return;
        }

        let (token, placeholder) = self.prepare_input(&mut tokens);

        // this route is already added
        if self.sub_routes.contains_key(&token) {
            // not the end of the full route, delegate the add route task to it's corresponding sub route.
            // compare with 1 because the first element is the token variable
            if tokens.len() > 1 {
                let route = self.sub_routes.get_mut(&token).unwrap();
                route.add(tokens.split_off(1), handler);
            } else {
                panic!("Route is added already, name: {}{}", self.path, token)
            }
        } else {
            let path = format!("{}{}/", self.path, token);
            let mut route;
            if tokens.len() == 1 {
                // only add handler at the end of the tokens vector
                route = Route::new(path, placeholder, Some(Box::new(handler)), HashMap::new());
            } else {
                route = Route::new(path, placeholder, None, HashMap::new());
                // will split at 1 panic? No, it panic if {at} > len()
                route.add(tokens.split_off(1), handler);
            };
            self.sub_routes.entry(token.to_string()).or_insert(route);
        }
    }

    fn handle(&self, request: &impl IRequest) -> Box<dyn IResponse> {
        let p = request.get_path_variables();
        if p.len() == 1 {
            let handler = self.handler.as_ref().unwrap();
            handler(request)
        } else {
            let r = Request{ method: request.get_method(), path_variables: request.get_path_variables().clone(), query_parameters: request.get_query_parameters().clone(), body: String::new()};
            let child_route = self.sub_routes.get(&p[1]).unwrap();
            child_route.handle(&r)
        }
    }
}

impl Display for Route {
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