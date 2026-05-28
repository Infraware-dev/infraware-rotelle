use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const FAIL_EVERY: u32 = 3;

pub struct IngressConflictScenario {
    request_count: Mutex<u32>,
}

impl IngressConflictScenario {
    pub fn new() -> Self {
        Self {
            request_count: Mutex::new(0),
        }
    }
}

impl Scenario for IngressConflictScenario {
    fn name(&self) -> &'static str {
        "ingress-conflict"
    }

    fn description(&self) -> &'static str {
        "Returns 502 on every 3rd GET / — simulates routing conflicts between a LoadBalancer Service and an Ingress controller."
    }

    fn activate(&self, _: &ActivationParams) {
        *self.request_count.lock().unwrap() = 0;
    }

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        let mut count = self.request_count.lock().unwrap();
        *count += 1;
        let n = *count;
        tracing::info!(count = n, "ingress-conflict: index access");

        if n.is_multiple_of(FAIL_EVERY) {
            tracing::warn!(count = n, "ingress-conflict: simulating 502 gateway error");
            IndexEffect::RespondWithStatus(502, gateway_error_html(n))
        } else {
            let next_fail = (n / FAIL_EVERY + 1) * FAIL_EVERY;
            IndexEffect::Respond(format!(
                "<p>Request <strong>{n}</strong>: routed correctly via LoadBalancer. \
                 Next conflict (502) at request <strong>{next_fail}</strong>.</p>"
            ))
        }
    }

    fn status_extras(&self) -> serde_json::Value {
        serde_json::json!({
            "request_count": *self.request_count.lock().unwrap(),
            "fail_every": FAIL_EVERY,
        })
    }
}

fn gateway_error_html(n: u32) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>502 Bad Gateway</title></head>
<body>
<h1>502 Bad Gateway</h1>
<p>The upstream server returned an invalid response.</p>
<p><small>Request {n} — simulated LoadBalancer + Ingress routing conflict</small></p>
</body>
</html>"#
    )
}
