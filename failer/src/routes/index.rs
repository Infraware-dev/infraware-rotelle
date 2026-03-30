use poem::{handler, web::{Data, Html}};
use crate::state::State;

#[handler]
pub fn index(state: Data<&State>) -> Html<String> {
    let case = state.failure_case.lock().unwrap();
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Failer</title></head>
<body>
<h1>Failer</h1>
<p>Current failure case: <strong>{case}</strong></p>
</body>
</html>"#
    ))
}
