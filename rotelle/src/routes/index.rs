use crate::state::State;
use poem::{
    handler,
    web::{Data, Html},
};
use tracing::info;

/// Number of index accesses between crashes in intermittent-01 mode.
const INTERMITTENT_01_CRASH_EVERY: u32 = 5;

#[handler]
pub fn index(state: Data<&State>) -> Html<String> {
    let case = state.failure_case.lock().unwrap().clone();

    let extra = if case == "intermittent-01" {
        let mut count = state.access_count.lock().unwrap();
        *count += 1;
        if (*count).is_multiple_of(INTERMITTENT_01_CRASH_EVERY) {
            info!(count = *count, "intermittent-01: simulating crash");
            std::process::exit(1);
        }
        format!(
            "<p>Crash every <strong>N={}</strong> accesses. Current count: <strong>{}</strong></p>",
            INTERMITTENT_01_CRASH_EVERY, *count
        )
    } else {
        String::new()
    };

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Rotelle</title></head>
<body>
<h1>Rotelle</h1>
<p>Current failure case: <strong>{case}</strong></p>
{extra}</body>
</html>"#
    ))
}
