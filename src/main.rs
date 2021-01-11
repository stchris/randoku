#![feature(proc_macro_hygiene, decl_macro)]

use rocket::response::status::BadRequest;

#[macro_use]
extern crate rocket;

#[get("/?<limit>")]
fn hello(limit: Option<u32>) -> Result<String, BadRequest<String>> {
    match limit {
        Some(l) => Ok(format!("Hello, limit {}!", l)),
        None => Err(BadRequest(Some(
            "limit should be an unsigned 32bit integer".to_string(),
        ))),
    }
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
