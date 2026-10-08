//! Resolve the model-call cap after the session's definition is selected.

use crate::agent::harness::definition::AgentDefinition;

pub(super) fn resolved_iteration_cap(
    agent_id: &str,
    definition: Option<&AgentDefinition>,
    global_cap: usize,
) -> usize {
    resolve_cap(agent_id, definition, global_cap, pioneer_runtime())
}

fn pioneer_runtime() -> bool {
    #[cfg(feature = "modules")]
    {
        crate::modules::computer_config::pioneer_local_runtime()
    }
    #[cfg(not(feature = "modules"))]
    {
        false
    }
}

fn resolve_cap(
    agent_id: &str,
    definition: Option<&AgentDefinition>,
    global_cap: usize,
    pioneer_runtime: bool,
) -> usize {
    let cap = definition.map_or(global_cap, AgentDefinition::effective_max_iterations);
    if pioneer_runtime && agent_id == "orchestrator" {
        cap.max(30)
    } else {
        cap
    }
}

#[cfg(test)]
#[path = "iteration_budget_tests.rs"]
mod tests;
