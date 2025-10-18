use crate::server::{Request, Response, Server};

pub mod server;
pub mod server_utils;

fn index(r: Request) -> Response {
    println!("{:#?}", r);
    Response("Hello".to_string())
}

fn main() {
    let mut server = Server::<Request, Response>::new();
    server.get("/".to_string(), index);
    server.post("/user/logout".to_string(), index);
    server.post("/user/login".to_string(), index);
    server.put("/user/:id/name".to_string(), index);
    server.delete("/book/chaper/:id".to_string(), index);
    server.run();
}
