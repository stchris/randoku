#![feature(proc_macro_hygiene, decl_macro)]

use rand::Rng;

#[macro_use]
extern crate rocket;

fn get_rand(from: Option<u32>, to: Option<u32>) -> u32 {
    let mut rng = rand::thread_rng();
    let from = match from {
        Some(f) => f,
        None => 0,
    };
    let to = match to {
        Some(f) => f,
        None => 100,
    };
    rng.gen_range(from..=to)
}

#[get("/")]
fn no_limit() -> String {
    format!("{}\n", get_rand(Some(0), Some(100)))
}

#[get("/<to>")]
fn upper_limit(to: u32) -> String {
    format!("{}\n", get_rand(Some(0), Some(to)))
}

#[get("/<from>/<to>")]
fn both_limits(from: u32, to: u32) -> String {
    format!("{}\n", get_rand(Some(from), Some(to)))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![no_limit, upper_limit, both_limits])
        .launch();
}
