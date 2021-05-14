#![feature(proc_macro_hygiene, decl_macro)]

use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;
use rocket::{request::FromRequest, Outcome, Rocket};

#[macro_use]
extern crate rocket;

struct UserAgent(Option<String>);

impl<'a, 'r> FromRequest<'a, 'r> for &'a UserAgent {
    type Error = ();

    fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        Outcome::Success(request.local_cache(|| {
            let value = request
                .headers()
                .get("User-Agent")
                .next()
                .map(|x| x.to_string());
            UserAgent(value)
        }))
    }
}

lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_rng(thread_rng()).unwrap());
}

fn get_rand(from: Option<u32>, to: Option<u32>) -> u32 {
    RNG.lock()
        .unwrap()
        .gen_range(from.unwrap_or(0)..=to.unwrap_or(100))
}

#[get("/")]
fn no_limit(user_agent: &UserAgent) -> String {
    if user_agent
        .0
        .as_ref()
        .unwrap_or(&"".to_string())
        .starts_with("curl/")
    {
        return "Hello curl!".to_string();
    }
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

fn rocket() -> Rocket {
    rocket::ignite().mount("/", routes![no_limit, upper_limit, both_limits])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::Client;

    #[test]
    fn test_random() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let req = client.get("/");
        let mut response = req.dispatch();

        let num: u32 = response
            .body_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 100);
    }

    #[test]
    fn test_upto() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let req = client.get("/3");
        let mut resp = req.dispatch();
        let num: u32 = resp
            .body_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 3);
    }

    #[test]
    fn test_from_to() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let req = client.get("/5/9");
        let mut resp = req.dispatch();
        let num: u32 = resp
            .body_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 9);
        assert!(num >= 5);
    }

    #[test]
    fn test_curl() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut req = client.get("/");
        req.add_header(rocket::http::hyper::header::UserAgent(
            "curl/1.1.1".to_string(),
        ));
        let mut response = req.dispatch();

        let resp = response.body_string().expect("a response");
        assert_eq!(resp, "Hello curl!");
    }
}
