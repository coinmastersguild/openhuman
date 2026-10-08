//! Pioneer authoring instructions derived from the currently registered tools.

pub(super) fn apply(archetype: &str, _pioneer: bool, _tool_names: &[&str]) -> String {
    archetype.to_owned()
}

#[cfg(test)]
#[path = "authoring_guidance_tests.rs"]
mod tests;
