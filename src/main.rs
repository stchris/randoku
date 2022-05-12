use std::collections::HashMap;
use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use rand::{thread_rng, Rng};

use lazy_static::lazy_static;
use rocket::request::{FromRequest, Outcome};
use rocket::response::content;
use rocket::response::status::BadRequest;
use rocket::{Build, Rocket};
use rocket_dyn_templates::handlebars::Handlebars;

#[macro_use]
extern crate rocket;

const INDEX: &str = include_str!("../templates/index.html");
struct UserAgentBrowser(());

#[rocket::async_trait]
/// A request guard which lets requests from browsers through or forwards otherwise.
/// Based on the info from MDN https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent
/// See also: https://github.com/stchris/randoku/issues/23
impl<'r> FromRequest<'r> for UserAgentBrowser {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let ua = *request.local_cache(|| {
            request
                .headers()
                .get("User-Agent")
                .next()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "".to_string())
                .starts_with("Mozilla/5.0")
        });
        match ua {
            true => Outcome::Success(UserAgentBrowser(())),
            _ => Outcome::Forward(()),
        }
    }
}

lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_rng(thread_rng()).unwrap());
}

fn get_rand(from: Option<u64>, to: Option<u64>) -> u64 {
    RNG.lock()
        .unwrap()
        .gen_range(from.unwrap_or(0)..=to.unwrap_or(100))
}

#[get("/", rank = 1)]
fn index_browser(_ua: UserAgentBrowser) -> content::RawHtml<String> {
    let context: HashMap<&str, &str> = HashMap::new();
    let reg = Handlebars::new();
    let res = reg
        .render_template(INDEX, &context)
        .expect("failed to render template");

    content::RawHtml(res)
}

#[get("/", rank = 2)]
fn index_plain() -> String {
    let num = get_rand(Some(0), Some(100));
    format!("{}\n", num)
}

#[get("/<to>")]
fn upper_limit(to: u64) -> String {
    format!("{}\n", get_rand(Some(0), Some(to)))
}

#[get("/<from>/<to>")]
fn both_limits(from: u64, to: u64) -> Result<String, BadRequest<String>> {
    if from > to {
        return Err(BadRequest::<_>(Some(format!(
            "Wrong parameter order: {} should be <= {} (try switching them around).",
            from, to
        ))));
    }
    Ok(format!("{}\n", get_rand(Some(from), Some(to))))
}

#[get("/shuffle/<list>")]
fn shuffle(list: String) -> String {
    let mut items: Vec<&str> = list.split(',').into_iter().collect();
    items.shuffle(&mut thread_rng());
    items.join("\n")
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount(
        "/",
        routes![
            index_plain,
            index_browser,
            upper_limit,
            both_limits,
            shuffle
        ],
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
        let mut req = client.get("/");
        req.add_header(Header::new("User-Agent", "Mozilla/5.0".to_string()));
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
        let num: u64 = resp
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 3);
    }

    #[test]
    fn test_upto_large_number() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/9999999999");
        let resp = req.dispatch();
        assert_eq!(resp.status().code, 200);

        let num: u64 = resp
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        dbg!(num);
    }

    #[test]
    fn test_from_to() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/5/9");
        let resp = req.dispatch();
        let num: u64 = resp
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 9);
        assert!(num >= 5);
    }

    #[test]
    fn test_from_greater_than_to_fails() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/9/5");
        let resp = req.dispatch();
        assert_eq!(resp.status().code, 400);
        assert_eq!(
            resp.into_string().unwrap().to_string(),
            "Wrong parameter order: 9 should be <= 5 (try switching them around).".to_string()
        );
    }

    #[test]
    fn test_random() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let mut req = client.get("/");
        req.add_header(Header::new("User-Agent", "curl/1.1.1".to_string()));
        let response = req.dispatch();

        let num: u64 = response
            .into_string()
            .expect("a response")
            .trim_end()
            .parse()
            .expect("a number");
        assert!(num <= 100);
    }

    #[test]
    fn test_shuffle() {
        let client = Client::untracked(rocket()).expect("valid rocket instance");
        let req = client.get("/shuffle/apples,bananas,oranges");
        let response = req.dispatch();

        let response = response.into_string().expect("a response");
        let response = response.trim_end();
        assert!(response.contains("apples"));
        assert!(response.contains("bananas"));
        assert!(response.contains("oranges"));
    }
}
