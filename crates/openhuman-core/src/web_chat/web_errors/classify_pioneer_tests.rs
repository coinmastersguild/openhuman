use super::*;

#[test]
fn pioneer_runtime_gateway_budget_never_prompts_for_a_cloud_plan() {
    for provider in ["custom_openai", "pioneer_local"] {
        let raw = format!(
            "{provider} API error (402 Payment Required): \
             {{\"error\":{{\"message\":\"token budget exhausted\",\"type\":\"gateway_error\"}}}}"
        );
        let value = classify_inference_error_for_runtime(&raw, true);
        assert_eq!(value.error_type, "budget_exhausted");
        assert_eq!(value.source, "provider");
        assert!(!value.retryable);
        assert_eq!(value.fallback_available, None);
        assert_eq!(value.provider.as_deref(), Some(provider));
        assert!(value.message.contains("Pioneer Studio"));
        assert!(value.message.contains("local"));
        assert!(value.message.contains("top up"));
        let lower = value.message.to_ascii_lowercase();
        for misleading in [
            "cloud",
            "managed",
            "plan",
            "tinyhumans",
            "use your own models",
            "api key",
        ] {
            assert!(
                !lower.contains(misleading),
                "{misleading}: {}",
                value.message
            );
        }
    }
}

#[test]
fn pioneer_budget_copy_is_scoped_to_the_actual_local_budget_signal() {
    let local = "custom_openai API error (402 Payment Required): token budget exhausted";
    let generic = classify_inference_error_for_runtime(local, false);
    assert!(generic.message.contains("managed (cloud)"));
    for raw in [
        "openrouter API error (402 Payment Required): requires more credits",
        "custom_openai API error (402 Payment Required): insufficient balance",
        "custom_openai API error (401 Unauthorized): token budget exhausted",
        "custom_openai API error (429 Too Many Requests): token budget exhausted",
    ] {
        let value = classify_inference_error_for_runtime(raw, true);
        assert!(
            !value.message.contains("Pioneer Studio"),
            "{raw}: {}",
            value.message
        );
    }
}
