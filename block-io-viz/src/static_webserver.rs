use std::net::SocketAddr;

use axum::{response::Html, response::Json, routing::get, Router};

#[derive(Clone, serde::Serialize)]
pub struct BlockDeviceInfo {
    pub name: String,
    pub size_sectors: u64,
}

pub async fn bind_and_serve(
    addr: SocketAddr,
    block_device: BlockDeviceInfo,
) -> Result<(), std::io::Error> {
    let block_json = Json(block_device);
    let app = Router::new()
        .route("/", get(root))
        .route("/app.js", get(app_js))
        .route("/block_device.json", get(|| async move { block_json }))
        .route("/styles.css", get(styles_css));
    //let addr: SocketAddr = "0.0.0.0:80".parse().unwrap();

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
