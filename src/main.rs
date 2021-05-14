use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;

lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_rng(thread_rng()).unwrap());
}

fn get_rand(from: Option<u32>, to: Option<u32>) -> u32 {
    RNG.lock()
        .unwrap()
        .gen_range(from.unwrap_or(0)..=to.unwrap_or(100))
}

#[get("/")]
async fn no_limit(req: HttpRequest) -> impl Responder {
    let user_agent = req.headers().get("User-Agent").unwrap().to_str().unwrap();
    if user_agent.starts_with("curl/") {
        return HttpResponse::Ok().body("Hello curl!".to_string());
    }
    HttpResponse::Ok().body(format!("{}\n", get_rand(Some(0), Some(100))))
}

#[get("/{to}")]
async fn upper_limit(web::Path(to): web::Path<u32>) -> impl Responder {
    format!("{}\n", get_rand(Some(0), Some(to)))
}

#[get("/{from}/{to}")]
async fn both_limits(web::Path((from, to)): web::Path<(u32, u32)>) -> impl Responder {
    format!("{}\n", get_rand(Some(from), Some(to)))
}

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .service(no_limit)
            .service(upper_limit)
            .service(both_limits),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::{Body, ResponseBody};
    use actix_web::{dev::Service, http, test};

    #[actix_rt::test]
    async fn test_index_ok() {
        let mut app = test::init_service(App::new().configure(app_config)).await;
        let req = test::TestRequest::with_header("user-agent", "curl/1.2.3")
            .uri("/")
            .to_request();
        let resp = app.call(req).await.unwrap();
        assert!(resp.status().is_success());

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => std::str::from_utf8(bytes).unwrap(),
            _ => panic!("Response error"),
        };
        assert_eq!(response_body, "Hello curl!");
    }
    /*
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
    #[actix_rt::test]
    async fn test_index_not_ok() {
        let req = test::TestRequest::default().to_http_request();
        let resp = index(req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }}

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
    */
}
