use poem::{Response, handler, http::StatusCode};

static LOGO: &[u8] = include_bytes!("../../static/logo.png");
static FAVICON: &[u8] = include_bytes!("../../static/favicon.png");

#[handler]
pub fn logo() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "image/png")
        .header("cache-control", "public, max-age=31536000, immutable")
        .body(LOGO.to_vec())
}

#[handler]
pub fn favicon() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "image/png")
        .header("cache-control", "public, max-age=31536000, immutable")
        .body(FAVICON.to_vec())
}
