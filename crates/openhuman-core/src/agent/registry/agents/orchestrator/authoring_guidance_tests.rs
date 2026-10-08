use super::*;

const ARCHETYPE: &str = include_str!("prompt.md");

#[test]
fn pioneer_runtime_authoring_routes_research_only_through_current_capabilities() {
    let text = apply(
        ARCHETYPE,
        true,
        &[
            "tool_search",
            "mcp_registry_tool_call",
            "web_fetch",
            "curl",
            "shell",
        ],
    );
    for absent in ["web_answer_tool", "web_search_tool", "web_contents_tool"] {
        assert!(
            !text.contains(absent),
            "unsupported native search route remains: {absent}"
        );
    }
    for required in [
        "public sources",
        "license",
        "attribution",
        "SHA256",
        "installed Blender",
        "material",
        "lighting",
        "composition",
        "preview",
        "blocking: true",
        "actual image",
        "refine",
        "motion",
        "final artifacts",
    ] {
        assert!(
            text.contains(required),
            "quality workflow missing {required}"
        );
    }
    assert!(text.contains("`web_fetch`"));
    assert!(text.contains("`curl`"));
    assert!(
        !text.contains("`browser`"),
        "a browser must not be invented"
    );
}

#[test]
fn pioneer_runtime_no_local_authoring_capability_is_not_advertised() {
    let text = apply(ARCHETYPE, true, &["web_fetch"]);
    assert!(!text.contains("web_answer_tool"));
    assert!(!text.contains("Pioneer visual authoring"));
    assert!(!text.contains("`curl`"));
}

#[test]
fn pioneer_runtime_generic_authoring_prompt_is_byte_identical() {
    assert_eq!(
        apply(
            ARCHETYPE,
            false,
            &["tool_search", "mcp_registry_tool_call", "shell"]
        ),
        ARCHETYPE
    );
}
