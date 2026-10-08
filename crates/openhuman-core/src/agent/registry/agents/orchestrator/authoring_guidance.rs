//! Pioneer authoring instructions derived from the currently registered tools.

pub(super) fn apply(archetype: &str, pioneer: bool, tool_names: &[&str]) -> String {
    if !pioneer {
        return archetype.to_owned();
    }
    let has = |name: &str| tool_names.contains(&name);
    let discovery = has("tool_search");
    let mcp = discovery && has("mcp_registry_tool_call");
    let mut out = archetype
        .lines()
        .filter(|line| !line.starts_with("- Web:"))
        .collect::<Vec<_>>()
        .join("\n");
    let routes = ["browser", "browser_open", "web_fetch", "curl"]
        .into_iter()
        .filter(|name| has(name))
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>();
    if !routes.is_empty() || mcp {
        out.push_str("\n\n## Pioneer public research\n\nRead public sources with the current available routes");
        if !routes.is_empty() {
            out.push_str(&format!(": {}", routes.join(", ")));
        }
        if mcp {
            out.push_str("; discover connected browser and asset tools with `tool_search`, using their exact returned schemas");
        }
        out.push_str(". Keep browser research bounded. A search provider not admitted by this runtime is unavailable; do not invent its tools or switch inference providers. After the direct web-read limit, use gathered references and continue local work; asset downloads from those sources still require the current tool's admission.\n");
    }
    if has("shell")
        || mcp
        || tool_names
            .iter()
            .any(|name| name.contains("blender") || name.contains("render_preview"))
    {
        out.push_str("\n## Pioneer visual authoring\n\nFor a polished visual or animation, research references and relevant techniques before choosing geometry; inspect the installed Blender version and the exact available tool schemas. Prefer suitable reusable models, materials, textures and environments over unsupported primitive stand-ins. Import only assets with a verified compatible license; record source URL, license, attribution requirements, local path and SHA256. Follow the admitted download/import tools; never execute scripts bundled with an untrusted asset. If no suitable asset is available, explain that limit and choose an intentional authored style.\n\nReserve working time and prepaid budget for a low-cost preview. Inspect an actual image from the rendered scene, critique silhouette/anatomy, material, lighting, composition and framing against the brief, then refine and render again. If image analysis is available, discover its exact schema and use `blocking: true` when critique gates this task; do not finalize with findings merely queued for a later turn. A missing or failed image analysis is not a visual review. Before final artifacts, verify the saved scene and requested media, actual frame count and motion using multiple frames or the video; a still preview alone cannot prove animation. Report verified output paths, asset provenance and remaining quality limits honestly.\n");
    }
    out
}

pub(super) fn pioneer_runtime() -> bool {
    #[cfg(feature = "modules")]
    {
        crate::modules::computer_config::pioneer_local_runtime()
    }
    #[cfg(not(feature = "modules"))]
    {
        false
    }
}

#[cfg(test)]
#[path = "authoring_guidance_tests.rs"]
mod tests;
