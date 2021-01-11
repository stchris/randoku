#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[get("/?<limit>")]
fn hello(limit: Option<u32>) -> String {
    let limit = limit.unwrap_or(100);
    format!("Hello, limit {}!", limit)
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
