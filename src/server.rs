use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

const SERVER_ADDRESS: &str = "127.0.0.1:8000";

pub trait IRequest {}
pub trait IResponse {}

pub struct Request;
pub struct Response;

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
    placeholder: String,
    handler: RequestHandler<T, V>,
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
            handle_connection(tcp_stream);
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
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    // let request_info: RequestInfo = RequestInfo::new(first_line);

    let content: Vec<String> = buf_reader
        .lines()
        .map(|value| value.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request content: {content:#?}");
    let request_data: Vec<_> = content[0].split('c').collect();

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

impl<T, V> RouteCollection<T, V> {
    fn new() -> Self {
        RouteCollection {
            routes: HashMap::new(),
        }
    }

    fn add(&mut self, path: String, handler: RequestHandler<T, V>) {
        self.routes
            .insert(path, Route::new("".to_string(), handler, HashMap::new()));
    }
}

impl<T, V> Route<T, V> {
    fn new(
        placeholder: String,
        handler: RequestHandler<T, V>,
        sub_routes: HashMap<String, Route<T, V>>,
    ) -> Self {
        Route {
            placeholder,
            handler,
            sub_routes,
        }
    }
}
