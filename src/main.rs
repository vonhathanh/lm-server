use crate::server::{IRequest, IResponse, Response, Server};

pub mod server;
pub mod server_utils;

// must return Box<T> because trait object is dynamically sized type, so we wrap it inside a smart pointer
fn index(r: &dyn IRequest) -> Box<dyn IResponse> {
    println!("{}", r.get_method());
    Box::new(Response("Hello world!".to_string()))
}

fn main() {
    let mut server = Server::new();
    server.get("/", index);
    server.post("/user/logout", index);
    server.post("/user/login", index);
    server.put("/user/:id/name", index);
    server.delete("/book/chaper/:id", index);
    server.run();
}
