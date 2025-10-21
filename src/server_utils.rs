use crate::server::Request;

pub fn parse_request(input: &String) -> Request {
        let request_data: Vec<&str> = input.split(' ').collect();
        let [method, path, _version] = request_data[..] else {
            panic!("invalid request")
        };

        let path_variables: Vec<String>;
        let query_parameters: Vec<(String, String)>;

        match path.find('?') {
            Some(idx) => {
                path_variables = parse_path_variables(&path[..idx]);
                query_parameters = parse_query_parameters(&path[idx+1..])
            },
            None => {
                path_variables = parse_path_variables(&path[..]);
                query_parameters = Vec::new();
            }
        }

        Request { method: method.to_string(), path_variables, query_parameters, body: "".to_string() }
    }

fn parse_path_variables(input: &str) -> Vec<String> {
    let strings: Vec<&str> = input.split('/').collect();
    return strings.iter().map(|s| format!("/{s}")).collect();
}

fn parse_query_parameters(input: &str) -> Vec<(String, String)> {
    let mut query_parameters = vec![];
    let query_parts: Vec<&str> = input.split('&').collect();

    for query in query_parts {
        let query_detail: Vec<&str> = query.split('=').collect();
        assert!(query_detail.len() == 2);
        query_parameters.push((query_detail[0].to_string(), query_detail[1].to_string()));
    }

    query_parameters
}