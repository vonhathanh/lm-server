use crate::server::{IRequest, IResponse, Response, Server};

pub mod server;
pub mod server_utils;

fn index(r: &dyn IRequest) -> Box<dyn IResponse> {
    println!("{}", r.get_method());
    Box::new(Response("Hello world!".to_string()))
}

fn main() {
    let mut server = Server::new();
    server.get("/".to_string(), index);
    server.post("/user/logout".to_string(), index);
    server.post("/user/login".to_string(), index);
    server.put("/user/:id/name".to_string(), index);
    server.delete("/book/chaper/:id".to_string(), index);
    server.run();
}
