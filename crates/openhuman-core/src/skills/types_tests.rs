use super::*;
use serde_json::json;

#[test]
fn an_mcp_result_maps_across_with_its_error_flag_intact() {
    let ok = tool_result_from_mcp(tinymcp_bus::McpToolResult {
        content: vec![tinymcp_bus::McpToolContent::Text {
            text: "fine".into(),
        }],
        is_error: false,
        markdown_formatted: None,
    });
    assert!(!ok.is_error);
    assert_eq!(ok.text(), "fine");

    let failed = tool_result_from_mcp(tinymcp_bus::McpToolResult {
        content: vec![tinymcp_bus::McpToolContent::Text {
            text: "boom".into(),
        }],
        is_error: true,
        markdown_formatted: Some("**boom**".into()),
    });
    assert!(failed.is_error);
    assert_eq!(failed.markdown_formatted.as_deref(), Some("**boom**"));
}

#[test]
fn pioneer_runtime_host_mcp_conversion_uses_the_same_bounded_adapter() {
    let result = tool_result_from_mcp(tinymcp_bus::McpToolResult::json(
        json!({"base64":"x".repeat(70 * 1024)}),
    ));
    assert!(!result.output().contains("xxxxx"));
    assert!(result.output().contains("bytes elided"));
    let result = tool_result_from_mcp(tinymcp_bus::McpToolResult {
        content: vec![tinymcp_bus::McpToolContent::Image {
            data: "bad".into(),
            mime_type: "image/png".into(),
        }],
        is_error: false,
        markdown_formatted: None,
    });
    assert!(result.is_error);
    assert!(result.follow_up.is_empty());
    assert!(!result.output().contains("bad"));
}

#[test]
fn pioneer_runtime_host_mcp_images_reach_the_native_follow_up_channel() {
    let data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGNgZGIGAAAOAAfXb+R4AAAAAElFTkSuQmCC";
    let result = tool_result_from_mcp(tinymcp::render_tool_result(&json!({"content":[
        {"type":"text","text":"preview"}, {"type":"image","mimeType":"image/png","data":data}
    ]})));
    assert!(!result.is_error);
    assert!(result.text().contains("preview"));
    assert_eq!(result.follow_up.len(), 2);
    assert!(
        matches!(&result.follow_up[1], ToolContent::Image { media_type, data: tinytools::ImageData::Base64(value) } if media_type == "image/png" && value == data)
    );
    assert!(!result.output().contains(data));
}
