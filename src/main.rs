use std::sync::Mutex;

use axum::extract::Path;
use axum::response::{Html, IntoResponse};
use axum::{headers::UserAgent, http::StatusCode, routing::get, Router, TypedHeader};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::{thread_rng, Rng};

use lazy_static::lazy_static;

const INDEX: &str = include_str!("../templates/index.html");

lazy_static! {
    static ref RNG: Mutex<StdRng> = Mutex::new(StdRng::from_rng(thread_rng()).unwrap());
}

fn get_rand(from: Option<u64>, to: Option<u64>) -> u64 {
    RNG.lock()
        .unwrap()
        .gen_range(from.unwrap_or(0)..=to.unwrap_or(100))
}

async fn root(TypedHeader(user_agent): TypedHeader<UserAgent>) -> impl IntoResponse {
    if user_agent.as_str().starts_with("Mozilla/5.0") {
        return Html(INDEX).into_response();
    }
    let num = get_rand(Some(0), Some(100));
    format!("{}\n", num).into_response()
}

async fn upper_limit(Path(to): Path<u64>) -> String {
    format!("{}\n", get_rand(Some(0), Some(to)))
}

enum ParamError {
    Order(u64, u64),
}

impl IntoResponse for ParamError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ParamError::Order(from, to) => (
                StatusCode::BAD_REQUEST,
                format!(
                    "Wrong parameter order: {} should be <= {} (try switching them around).",
                    from, to
                ),
            )
                .into_response(),
        }
    }
}

async fn both_limits(Path(limits): Path<(u64, u64)>) -> Result<String, ParamError> {
    let (from, to) = limits;
    if from > to {
        return Err(ParamError::Order(from, to));
    }
    Ok(format!("{}\n", get_rand(Some(from), Some(to))))
}

async fn shuffle(Path(list): Path<String>) -> String {
    let mut items: Vec<&str> = list.split(',').collect();
    items.shuffle(&mut thread_rng());
    items.join("\n")
}

fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/:from", get(upper_limit))
        .route("/:from/*to", get(both_limits))
        .route("/shuffle/:list", get(shuffle))
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    Ok(app().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_index() {
        let app = app();
        let resp = app
            .oneshot(
                Request::builder()
                    .header("User-Agent", "Mozilla/5.0")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("Content-Type").unwrap(),
            "text/html; charset=utf-8"
        );
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("Randoku"));
    }

    #[tokio::test]
    async fn test_upto() {
        let app = app();
        let resp = app
            .oneshot(Request::builder().uri("/3").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let num: u64 = body.trim_end().parse().expect("a number");
        assert!(num <= 3);
    }

    #[tokio::test]
    async fn test_upto_large_number() {
        let app = app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/9999999999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let _: u64 = body.trim_end().parse().expect("a number");
    }

    #[tokio::test]
    async fn test_from_to() {
        let app = app();
        let resp = app
            .oneshot(Request::builder().uri("/5/9").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let num: u64 = body.trim_end().parse().expect("a number");
        assert!(num <= 9);
        assert!(num >= 5);
    }

    #[tokio::test]
    async fn test_from_greater_than_to_fails() {
        let app = app();
        let resp = app
            .oneshot(Request::builder().uri("/9/5").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(
            body,
            "Wrong parameter order: 9 should be <= 5 (try switching them around).".to_string()
        );
    }

    #[tokio::test]
    async fn test_random() {
        let app = app();
        let resp = app
            .oneshot(
                Request::builder()
                    .header("User-Agent", "curl/1.1.1")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("Content-Type").unwrap(),
            "text/plain; charset=utf-8"
        );
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let num: u64 = body.trim_end().parse().unwrap();
        assert!(num <= 100);
    }

    #[tokio::test]
    async fn test_shuffle() {
        let app = app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/shuffle/apples,bananas,oranges")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("apples"));
        assert!(body.contains("bananas"));
        assert!(body.contains("oranges"));
    }
}
