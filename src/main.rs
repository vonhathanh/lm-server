use crate::server::{Request, Response, Server};

pub mod server;

fn index(r: Request) -> Response {
    !todo!()
}

fn main() {
    let mut server = Server::<Request, Response>::new();
    server.get("/".to_string(), index);
    server.post("/user/hanhvn".to_string(), index);
    server.put("/user?age=10&occupation=developer".to_string(), index);
    server.delete("/book/chaper/12".to_string(), index);
    server.run();
}
