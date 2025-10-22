use serde::{Deserialize};
use crate::server::{IRequest, IResponse, Response};

#[derive(Deserialize)]
struct Credentical {
    username: String,
    password: String,
}

pub fn login(r: &dyn IRequest) -> Box<dyn IResponse> {
    let cred: Credentical = serde_json::from_str(&r.get_body()).expect("Invalid data format, expect {'username': string, 'password': string}");
    let response = format!("username: {}, password: {}", cred.username, cred.password);
    println!("{}", response);
    Box::new(Response(response))
}