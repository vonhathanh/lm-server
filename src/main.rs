use crate::server::{HttpStatus, IRequest, IResponse, Response, Server};

// TODO: mod vs pub mod
mod auth;
mod server;
mod server_utils;
mod route;

// must return Box<T> because trait object is dynamically sized type, so we wrap it inside a smart pointer
fn index(_r: &dyn IRequest) -> Box<dyn IResponse> {
    Box::new(Response::new(
        "Hello world!".to_string(),
        HttpStatus::OK.to_string(),
    ))
}

fn main() {
    let mut server = Server::new();
    server.get("/", index);
    server.post("/user/logout", index);
    server.post("/user/login", auth::login);
    server.put("/user/:id/name", index);
    server.delete("/book/chaper/:id", index);
    server.run();
}
