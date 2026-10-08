use super::*;

#[test]
fn pioneer_runtime_image_critique_waits_inside_the_current_turn_by_default() {
    assert!(blocking_for(
        "analyze_image",
        &json!({"prompt": "critique the current render before finalizing"}),
        true
    ));
    assert!(!blocking_for(
        "analyze_image",
        &json!({"prompt": "inspect"}),
        false
    ));
    assert!(!blocking_for(
        "delegate_researcher",
        &json!({"prompt": "research"}),
        true
    ));
}

#[test]
fn pioneer_runtime_image_analysis_explicit_background_choice_is_preserved() {
    assert!(!blocking_for(
        "analyze_image",
        &json!({"blocking": false}),
        true
    ));
    assert!(blocking_for(
        "analyze_image",
        &json!({"blocking": true}),
        false
    ));
    assert!(blocking_for(
        "delegate_researcher",
        &json!({"blocking": true}),
        true
    ));
}
