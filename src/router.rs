use axum::response::Html;
use axum::Router;
use axum::routing::get;

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

pub fn register_router() -> Router {
    let r = Router::new()
        .route("/", get(handler));
    r
}