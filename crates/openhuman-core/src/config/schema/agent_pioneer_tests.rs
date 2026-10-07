use super::*;

#[test]
fn pioneer_runtime_image_uses_native_tools_and_preserves_developer_default() {
    assert_eq!(default_tool_dispatcher_for_image_pin(true), "native");
    assert_eq!(default_tool_dispatcher_for_image_pin(false), "python");
}

#[test]
fn pioneer_runtime_default_matches_image_build_pin() {
    let expected =
        default_tool_dispatcher_for_image_pin(option_env!("PIONEER_TINYCOMPUTER_SHA256").is_some());
    assert_eq!(AgentConfig::default().tool_dispatcher, expected);
    let deserialized: AgentConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(deserialized.tool_dispatcher, expected);
}
