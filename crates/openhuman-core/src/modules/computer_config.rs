//! The private configuration OpenHuman hands the TinyComputer module.
//!
//! It is delivered only through the module lifecycle callback
//! (`reinitialize_module`), never logged, and rebuilt on every call so a
//! rotated or revoked credential takes effect at once:
//!
//! - `jev` — the decision model: Jev through TinyHumans when signed in, or
//!   the user's OpenRouter key; OpenJev or Sage with their own keys.
//! - `planner` — the planner, rescue and output models: through the
//!   TinyHumans proxy when signed in, else the user's OpenRouter key.
//! - `browser` — the Chrome executable the user picked, if any.

use serde_json::{json, Map, Value};

use crate::config::{Config, DecisionModel};

/// Trusted host switch. Agent config cannot change the fixed local routes.
pub(crate) fn pioneer_local_runtime() -> bool {
    matches!(
        std::env::var("PIONEER_LOCAL_RUNTIME").as_deref(),
        Ok("1" | "true")
    )
}

fn pioneer_config(config: &Config, api_key: Option<&str>) -> Value {
    let Some(api_key) = api_key.filter(|key| !key.trim().is_empty()) else {
        // Missing scoped identity fails closed; never use a hosted fallback.
        return json!({});
    };
    let mut value = json!({
        "jev": {"provider": "pioneer_local", "api_key": api_key,
            "endpoint_url": "http://10.88.0.1:12500/v1/systemone", "model": "analytic-latest",
            "timeout_ms": 6000, "max_retries": 1},
        "planner": {"provider": "pioneer_local", "api_key": api_key,
            "endpoint_url": "http://10.88.0.1:12500/v1", "model": "glm-5.3-flash-local",
            "rescue_model": "glm-5.3-flash-local", "output_model": "glm-5.3-flash-local"},
        "browser": {"headless": false, "perception": "tree", "args": ["--no-sandbox", "--disable-dev-shm-usage", "--proxy-server=http://10.88.0.1:3128", "--proxy-bypass-list=localhost;127.0.0.1;[::1]"]}
    });
    if let Some(executable) = config.browser.chrome_path.as_ref() {
        value["browser"]["executable"] = json!(executable);
    }
    value
}
use crate::inference::provider::factory::lookup_key_for_slug;
use crate::security::credentials::session_support::{
    is_local_session_token, resolve_backend_credential, BackendCredential,
};

/// A stored provider key, falling back to its environment variable.
fn provider_key(config: &Config, slug: &str, env: &str) -> Option<String> {
    lookup_key_for_slug(slug, config)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| std::env::var(env).ok().filter(|key| !key.trim().is_empty()))
}

/// The signed-in TinyHumans bearer, if any. The offline local token is not
/// a backend credential.
fn tinyhumans_bearer(config: &Config) -> Option<String> {
    resolve_backend_credential(config)
        .ok()
        .map(BackendCredential::into_secret)
        .filter(|token| !is_local_session_token(token))
}

fn jev(config: &Config, hosted: Option<&str>) -> Option<Value> {
    match config.computer.decision_model {
        DecisionModel::Jev => match hosted {
            Some(api_key) => Some(json!({
                "api_key": api_key,
                "provider": "tiny_humans_open_router",
                "sdk_name": crate::backend::product_identity()
            })),
            // A headless or BYOK host may have no TinyHumans session. Direct
            // OpenRouter Jev remains usable with its own scoped credential.
            None => provider_key(config, "openrouter", "OPENROUTER_API_KEY")
                .map(|api_key| json!({ "api_key": api_key, "provider": "open_router" })),
        },
        DecisionModel::OpenJev => provider_key(config, "openjev", "OPENJEV_API_KEY")
            .map(|api_key| json!({ "api_key": api_key, "provider": "open_jev" })),
        DecisionModel::Sage => provider_key(config, "sage", "SAGE_API_KEY").map(|api_key| {
            json!({ "api_key": api_key, "provider": "sage", "fast": config.computer.sage_fast })
        }),
    }
}

fn planner(config: &Config, hosted: Option<&str>) -> Option<Value> {
    let mut planner = match hosted {
        Some(api_key) => json!({
            "api_key": api_key,
            "provider": "tiny_humans",
            "sdk_name": crate::backend::product_identity()
        }),
        None => {
            let api_key = provider_key(config, "openrouter", "OPENROUTER_API_KEY")?;
            json!({ "api_key": api_key, "provider": "open_router" })
        }
    };
    let computer = &config.computer;
    for (key, value) in [
        ("model", &computer.planner_model),
        ("rescue_model", &computer.rescue_model),
    ] {
        if let Some(model) = value {
            planner[key] = json!(model);
        }
    }
    Some(planner)
}

/// Build the module configuration for `config`.
#[must_use]
pub fn module_config(config: &Config) -> Value {
    if pioneer_local_runtime() {
        return pioneer_config(config, std::env::var("MODEL_API_KEY").ok().as_deref());
    }
    let hosted = tinyhumans_bearer(config);
    let mut out = Map::new();
    if let Some(jev) = jev(config, hosted.as_deref()) {
        out.insert("jev".into(), jev);
    }
    if let Some(planner) = planner(config, hosted.as_deref()) {
        out.insert("planner".into(), planner);
    }
    if let Some(executable) = config.browser.chrome_path.as_ref() {
        out.insert("browser".into(), json!({ "executable": executable }));
    }
    tracing::debug!(
        decision_model = config.computer.decision_model.as_str(),
        jev = out.contains_key("jev"),
        planner = out.contains_key("planner"),
        hosted = hosted.is_some(),
        "[computer] module config built"
    );
    Value::Object(out)
}

/// Which account pays for TinyComputer's decisions, for the settings UI:
/// `hosted` (TinyHumans credits), `direct_openrouter` (the user's key),
/// `open_jev` or `sage` (their own keys), or `unavailable` (no credential).
#[must_use]
pub fn billing_route(config: &Config) -> &'static str {
    match module_config(config)["jev"]["provider"].as_str() {
        Some("tiny_humans_open_router") => "hosted",
        Some("open_router") => "direct_openrouter",
        Some("open_jev") => "open_jev",
        Some("sage") => "sage",
        Some("pioneer_local") => "pioneer_local",
        _ => "unavailable",
    }
}

#[cfg(test)]
#[path = "computer_config_tests.rs"]
mod tests;
