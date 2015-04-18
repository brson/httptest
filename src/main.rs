#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate serde;

use iron::prelude::*;
use iron::status;
use serde::json;

#[derive(Serialize, Deserialize)]
struct Greeting {
    msg: String
}

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        let greeting = Greeting { msg: "Hello, World".to_string() };
        let payload = json::to_string(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    Iron::new(hello_world).http("localhost:3000").unwrap();
    println!("On 3000");
}
