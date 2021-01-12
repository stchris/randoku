#![feature(proc_macro_hygiene, decl_macro)]

use std::sync::{Mutex};

use rand::{SeedableRng};
use rand::rngs::StdRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;


#[macro_use]
extern crate rocket;

lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_rng(thread_rng()).unwrap());
}

fn get_rand(from: Option<u32>, to: Option<u32>) -> u32 {
    RNG.lock().unwrap().gen_range(from.unwrap_or(0)..=to.unwrap_or(100))
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
