#![feature(proc_macro_hygiene, decl_macro)]

use rand::{SeedableRng};
use rand::rngs::StdRng;
use rand::Rng;

use rocket::State;

#[macro_use]
extern crate rocket;

struct Data {
    rng: StdRng,
}

fn get_rand(from: Option<u32>, to: Option<u32>, data: State<Data>) -> u32 {
    let from = match from {
        Some(f) => f,
        None => 0,
    };
    let to = match to {
        Some(f) => f,
        None => 100,
    };
    data.rng.clone().gen_range(from..=to)
}

#[get("/")]
fn no_limit(data: State<Data>) -> String {
    format!("{}\n", get_rand(Some(0), Some(100), data))
}

#[get("/<to>")]
fn upper_limit(to: u32, data: State<Data>) -> String {
    format!("{}\n", get_rand(Some(0), Some(to), data))
}

#[get("/<from>/<to>")]
fn both_limits(from: u32, to: u32, data: State<Data>) -> String {
    format!("{}\n", get_rand(Some(from), Some(to), data))
}

fn main() {
    let rng = StdRng::from_entropy();
    rocket::ignite()
        .mount("/", routes![no_limit, upper_limit, both_limits])
        .manage(Data{rng})
        .launch();
}
