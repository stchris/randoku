use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;
use rocket::request::{FromRequest, Outcome};
use rocket::{Build, Rocket};

use askama::Template;

#[macro_use]
extern crate rocket;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

struct UserAgentCurl(());

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgentCurl {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let ua = *request.local_cache(|| {
            request
                .headers()
                .get("User-Agent")
                .next()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "".to_string())
                .starts_with("curl/")
        });
        match ua {
            true => Outcome::Success(UserAgentCurl(())),
            _ => Outcome::Forward(()),
        }
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

#[get("/", rank = 1)]
fn index_plain(_ua: UserAgentCurl) -> String {
    let num = get_rand(Some(0), Some(100));
    format!("{}", num)
}

#[get("/", rank = 2)]
fn index_browser() -> IndexTemplate {
    IndexTemplate {}
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
    rocket::build().mount(
        "/",
        routes![index_plain, index_browser, upper_limit, both_limits],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Header;
    use rocket::local::blocking::Client;

    #[test]
    fn test_index() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/");
        let response = req.dispatch();
        let content = response
            .headers()
            .iter()
            .find(|h| h.name() == "Content-Type")
            .unwrap();
        let content = content.value();
        assert!(content.contains("text/html"));
        assert!(response.into_string().unwrap().contains("Randoku"));
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
    fn test_random() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let mut req = client.get("/");
        req.add_header(Header::new("User-Agent", "curl/1.1.1".to_string()));
        let response = req.dispatch();

        let num: u32 = response
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 100);
    }
}
