//! Shared tool result types used by the tool and node runtime surfaces.
//!
//! The definitions live in [`tinytools`]; this module is the stable host import
//! path for the ~14 call sites that already name it, and the home of the one
//! conversion that is genuinely ours.

pub use tinytools::{ToolContent, ToolResult};

/// Converts a rendered MCP result into this application's tool result.
///
/// The two shapes are the same by construction — the module's was derived from
/// this one when the client was extracted — so this is a mapping, not a
/// translation.
///
/// It is a free function rather than a `From` impl because both types are now
/// foreign to this crate: [`ToolResult`] belongs to `tinytools` and
/// `McpToolResult` to `tinymcp_bus`, which the orphan rule forbids us from
/// bridging with a trait impl. The reason the conversion is written **once**
/// has not changed with its shape: spelled out at each call site, it would be
/// as many chances to get the error flag the wrong way round.
pub fn tool_result_from_mcp(result: tinymcp_bus::McpToolResult) -> ToolResult {
    tinymcp::tools::tool_result(result)
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
