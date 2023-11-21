use std::net::SocketAddr;

use axum::{response::Html, response::Json, routing::get, Router};

#[derive(Clone, serde::Serialize)]
pub struct InitializationData {
    pub name: String,
    pub size_sectors: u64,
    pub websocket_port: u16,
}

pub async fn bind_and_serve(
    addr: SocketAddr,
    initialization_data: InitializationData,
) -> Result<(), std::io::Error> {
    let initialization_json = Json(initialization_data);
    let app = Router::new()
        .route("/", get(root))
        .route("/app.js", get(app_js))
        .route(
            "/block_device.json",
            get(|| async move { initialization_json }),
        )
        .route("/styles.css", get(styles_css));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn root() -> Html<&'static str> {
    Html(include_str!("../../block-io-viz-webapp/index.html"))
}
async fn app_js() -> Html<&'static str> {
    Html(include_str!("../../block-io-viz-webapp/dist/bundle.js"))
}
async fn styles_css() -> Html<&'static str> {
    Html(include_str!("../../block-io-viz-webapp/css/styles.css"))
}
