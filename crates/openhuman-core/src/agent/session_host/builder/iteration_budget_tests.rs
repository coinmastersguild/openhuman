use super::*;
use crate::agent::harness::definition::{AgentDefinitionRegistry, IterationPolicy};

fn builtin(id: &str) -> AgentDefinition {
    AgentDefinitionRegistry::builtins_only()
        .get(id)
        .cloned()
        .expect("known built-in definition")
}

#[test]
fn pioneer_runtime_orchestrator_has_room_for_authoring_before_wrap_up() {
    let definition = builtin("orchestrator");
    assert_eq!(definition.effective_max_iterations(), 15);
    // The selected definition, not an unrelated global override, is resolved
    // before the trusted runtime's floor is applied.
    for global_cap in [1, 10, 100] {
        assert_eq!(
            resolve_cap("orchestrator", Some(&definition), global_cap, true),
            30
        );
    }
}

#[test]
fn pioneer_runtime_generic_and_other_agents_keep_their_definition_caps() {
    let definition = builtin("orchestrator");
    assert_eq!(
        resolve_cap("orchestrator", Some(&definition), 100, false),
        15
    );
    let critic = builtin("critic");
    assert_eq!(resolve_cap("critic", Some(&critic), 100, true), 5);
    assert_eq!(resolve_cap("unknown-worker", None, 7, true), 7);
    assert_eq!(resolve_cap("orchestrator", None, 7, false), 7);

    let mut custom = definition;
    custom.max_iterations = 3;
    custom.iteration_policy = IterationPolicy::Strict;
    assert_eq!(resolve_cap("orchestrator", Some(&custom), 100, false), 3);
}

#[test]
fn pioneer_runtime_floor_does_not_reduce_a_larger_declared_cap() {
    let mut definition = builtin("orchestrator");
    definition.max_iterations = 40;
    assert_eq!(resolve_cap("orchestrator", Some(&definition), 1, true), 40);
    definition.max_iterations = 15;
    definition.iteration_policy = IterationPolicy::Extended;
    assert_eq!(resolve_cap("orchestrator", Some(&definition), 1, true), 50);
}

#[test]
fn pioneer_runtime_resolved_budget_reaches_existing_bounded_policy() {
    let definition = builtin("orchestrator");
    let cap = resolve_cap("orchestrator", Some(&definition), 100, true);
    let actual = crate::agent::tinyagents::run_policy_for(cap, false);
    let previous = crate::agent::tinyagents::run_policy_for(15, false);
    assert_eq!(actual.limits.max_model_calls, 30);
    assert_eq!(actual.limits.max_tool_calls, 240);
    assert_eq!(
        actual.limits.max_wall_clock_ms,
        previous.limits.max_wall_clock_ms
    );
    assert_eq!(
        actual.limits.max_model_call_ms,
        previous.limits.max_model_call_ms
    );
    assert_eq!(actual.limits.max_depth, previous.limits.max_depth);
    assert_eq!(
        actual.limits.max_retries_per_call,
        previous.limits.max_retries_per_call
    );
}
