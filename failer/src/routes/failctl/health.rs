use poem::handler;

#[handler]
pub fn health() -> &'static str {
    "ok"
}
