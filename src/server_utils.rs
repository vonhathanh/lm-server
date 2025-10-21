use crate::server::Request;

pub fn parse_request(input: &String) -> Request {
    let request_data: Vec<&str> = input.split(' ').collect();
    let [method, path, _version] = request_data[..] else {
        panic!("invalid request")
    };

    let query: Vec<(String, String)>;

    match path.find('?') {
        Some(idx) => query = parse_query(&path[idx + 1..]),
        None => {
            query = Vec::new();
        }
    }

    Request {
        method: method.to_string(),
        path: path.to_string(),
        query,
        body: "".to_string(),
    }
}

fn parse_query(input: &str) -> Vec<(String, String)> {
    let mut query = vec![];
    let query_parts: Vec<&str> = input.split('&').collect();

    for raw_query in query_parts {
        let query_tuple: Vec<&str> = raw_query.split('=').collect();
        assert!(query_tuple.len() == 2);
        query.push((query_tuple[0].to_string(), query_tuple[1].to_string()));
    }

    query
}
