use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;
use rocket::request::{FromRequest, Outcome};
use rocket::{Build, Rocket};

#[macro_use]
extern crate rocket;

#[derive(Debug, Clone, Copy)]
enum UserAgent {
    Cli,
    Browser,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(*request.local_cache(|| {
            let value = request
                .headers()
                .get("User-Agent")
                .next()
                .map(|x| x.to_string());
            match value {
                Some(value) if value.starts_with("curl/") => UserAgent::Cli,
                _ => UserAgent::Browser,
            }
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
fn no_limit(user_agent: UserAgent) -> String {
    match user_agent {
        UserAgent::Cli => "Hello curl!".to_string(),
        UserAgent::Browser => format!("{}\n", get_rand(Some(0), Some(100))),
    }
}

#[get("/<to>")]
fn upper_limit(to: u32) -> String {
    format!("{}\n", get_rand(Some(0), Some(to)))
}

#[get("/<from>/<to>")]
fn both_limits(from: u32, to: u32) -> String {
    format!("{}\n", get_rand(Some(from), Some(to)))
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![no_limit, upper_limit, both_limits])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Header;
    use rocket::local::blocking::Client;

    #[test]
    fn test_random() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/");
        let response = req.dispatch();

        let num: u32 = response
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 100);
    }

    #[test]
    fn test_upto() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/3");
        let resp = req.dispatch();
        let num: u32 = resp
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 3);
    }

    #[test]
    fn test_from_to() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/5/9");
        let resp = req.dispatch();
        let num: u32 = resp
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 9);
        assert!(num >= 5);
    }

    #[test]
    fn test_curl() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let mut req = client.get("/");
        req.add_header(Header::new("User-Agent", "curl/1.1.1".to_string()));
        let response = req.dispatch();

        let resp = response.into_string().expect("a response");
        assert_eq!(resp, "Hello curl!");
    }
}
