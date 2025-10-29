use crate::server::RequestHandler;

pub struct Route {
    handler: Option<RequestHandler>,
    children: [Option<Box<Route>>; 26]
}

impl Route {
    pub fn new(handler: Option<RequestHandler>) -> Self {
        Route { handler, children: Default::default() }
    }

    pub fn insert(&mut self, path: String, handler: Option<RequestHandler>) {
        // record the first character {c} to chilren array
        // find {c} index by c - 'a'
        // if {c} == '/' then c.index = 26
        // if {c} == ':' then c.index = 27
        // if c - 'a' not between [0, 25] then throw error
        // children[index] = new Route() if children[index] is not None
        // call insert(path[1..], handler)
        // if path.len() == 1 then self.handler = handler
        // can we remove the recursive call?
        let mut next: &Route = self;
        for c in path.chars() {
            let mut idx: i32 = -1;
            if c == '/' {
                idx = 26;
            } else if c == ':' {
                idx = 27
            } else {
                idx = c as i32 - 65;
            }
            if idx < 0 || idx > 27 {
                panic!("Invalid character: {c} found in request path")
            }

            match &self.children[idx as usize] {
                Some(route) => next = route,
                None => {
                    let node = Box::new(Route::new(None));
                    
                },
            }
        }

    }
}