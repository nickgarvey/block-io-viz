use std::net::SocketAddr;

use axum::{response::Html, routing::get, Router};

pub async fn bind_and_serve() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", get(root))
        .route("/app.js", get(app_js));
    let addr: SocketAddr = "0.0.0.0:80".parse().unwrap();

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn root() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}
async fn app_js() -> Html<&'static str> {
    Html(include_str!("../static/app.js"))
}
