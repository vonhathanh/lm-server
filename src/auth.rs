use crate::server::{IRequest, IResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct Credentical {
    username: String,
    password: String,
}

pub fn login(r: &dyn IRequest) -> Box<dyn IResponse> {
    match serde_json::from_str::<Credentical>(&r.get_body()) {
        Ok(cred) => {
            let response = format!("username: {}, password: {}", cred.username, cred.password);
            println!("{}", response);
            Box::new(Response(response))
        },
        Err(_) => {
            Box::new(Response("Invalid data format, expect {'username': string, 'password': string}".to_string()))
        },
    }
}
