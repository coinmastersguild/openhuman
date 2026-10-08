//! Exercise the real durable codec, host builder policy and executable guard.
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use tinyagents_runtime::{
    DriverFailure, DriverOutcome, DriverRequest, ResumeMode, SessionDriver, SessionTurnRequest,
    ToolSnapshot, TurnOptions,
};
use tinyagents_session::transcript::{FileTranscriptLocator, SessionRef, TranscriptMeta};
use tinymcp::tools::{McpToolInvoker, McpToolSource};
use tinymcp_bus::{ConnectedServerOverview, McpTool, McpToolContent, McpToolResult};
use tinytools::{Tool, ToolSpec};

use super::{ensure_snapshot_tools_are_executable, OpenHumanRunContext};
use crate::agent::session_host::{
    runtime_session::session_builder_for_runtime, OpenHumanTranscriptCodec,
};

#[derive(Debug, Default)]
struct Invocations(Mutex<Vec<String>>);

#[async_trait]
impl McpToolInvoker for Invocations {
    async fn invoke(
        &self,
        server: &str,
        _: &str,
        _: serde_json::Value,
    ) -> tinymcp::Result<McpToolResult> {
        self.0.lock().unwrap().push(server.to_owned());
        Ok(McpToolResult {
            content: vec![McpToolContent::Text {
                text: "sunny".into(),
            }],
            ..Default::default()
        })
    }
}

fn installed_tools(server_id: &str, invocations: &Arc<Invocations>) -> Arc<Vec<Box<dyn Tool>>> {
    // These are real tinymcp names: reinstalling under a new UUID changes the suffix.
    let source = McpToolSource::from_overview(&ConnectedServerOverview {
        server_id: server_id.into(),
        qualified_name: "example/weather".into(),
        display_name: "Weather".into(),
        description: None,
        instructions: None,
        tools: vec![McpTool {
            name: "forecast".into(),
            description: Some("Forecast".into()),
            input_schema: json!({"type":"object","properties":{}}),
        }],
    });
    let invoker: Arc<dyn McpToolInvoker> = invocations.clone();
    Arc::new(
        tinymcp::tools::tools_for(&[source], &invoker)
            .into_iter()
            .map(|tool| Box::new(tool) as Box<dyn Tool>)
            .collect(),
    )
}

fn snapshot(tools: &[Box<dyn Tool>]) -> ToolSnapshot {
    ToolSnapshot::new(
        tools
            .iter()
            .map(|tool| ToolSpec {
                name: tool.name().into(),
                description: tool.description().into(),
                parameters: tool.parameters_schema(),
            })
            .collect(),
    )
    .unwrap()
}

struct GuardedDriver {
    tools: Arc<Vec<Box<dyn Tool>>>,
    visible: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl SessionDriver<OpenHumanRunContext> for GuardedDriver {
    async fn execute(
        &self,
        request: DriverRequest<OpenHumanRunContext>,
    ) -> Result<DriverOutcome, DriverFailure> {
        let names = request
            .tools
            .specs()
            .iter()
            .map(|tool| tool.name.clone())
            .collect();
        // Exactly the production guard; no bypass or replacement registry.
        ensure_snapshot_tools_are_executable(&names, &self.tools, &Arc::new(Vec::new()))?;
        *self.visible.lock().unwrap() = request
            .tools
            .specs()
            .iter()
            .map(|tool| tool.name.clone())
            .collect();
        for tool in self.tools.iter().filter(|tool| names.contains(tool.name())) {
            assert!(!tool.execute(json!({})).await.unwrap().is_error);
        }
        let mut history = request.history;
        history.push(tinyinference_llm::message::Message::assistant(
            "weather checked",
        ));
        Ok(DriverOutcome {
            history,
            output: Some("weather checked".into()),
            partial: None,
            interrupted: false,
        })
    }
}

fn metadata() -> TranscriptMeta {
    TranscriptMeta {
        agent_name: "weather-agent".into(),
        agent_id: Some("weather-agent".into()),
        agent_type: Some("root".into()),
        dispatcher: "test".into(),
        provider: None,
        model: None,
        created: "2026-01-01T00:00:00Z".into(),
        updated: "2026-01-01T00:00:00Z".into(),
        turn_count: 0,
        prefix_message_count: None,
        input_tokens: 0,
        output_tokens: 0,
        cached_input_tokens: 0,
        charged_amount_usd: 0.0,
        thread_id: None,
        task_id: None,
        session_id: None,
        parent_session_id: None,
    }
}

fn options(resume: ResumeMode) -> TurnOptions<OpenHumanRunContext> {
    let context = OpenHumanRunContext::new();
    TurnOptions {
        request_id: None,
        thread_id: Some("weather-conversation".into()),
        stream: false,
        resume,
        session: None,
        cancellation: context.cancellation.clone(),
        run_context: context
            .into_tinyagents(tinyagents_harness::context::RunConfig::new("weather")),
    }
}

async fn restored_session(pioneer: bool, removed: bool) {
    let root = tempfile::tempdir().unwrap();
    let locator = Arc::new(FileTranscriptLocator::new(root.path()));
    let conversation = SessionRef::root("weather-conversation");
    let invocations = Arc::new(Invocations::default());
    let old_id = "11111111-1111-4111-8111-111111111111";
    let new_id = "22222222-2222-4222-8222-222222222222";
    let old_tools = installed_tools(old_id, &invocations);
    let old_name = old_tools[0].name().to_owned();
    let mut first = session_builder_for_runtime(
        Arc::new(GuardedDriver {
            tools: old_tools.clone(),
            visible: Arc::new(Mutex::new(Vec::new())),
        }),
        pioneer,
    )
    .codec(Arc::new(OpenHumanTranscriptCodec))
    .tool_snapshot(snapshot(&old_tools))
    .session(locator.clone(), conversation.clone(), metadata())
    .build()
    .unwrap();
    let prior = first
        .turn(
            SessionTurnRequest::new(tinyinference_llm::message::Message::user(
                "Remember the forecast",
            )),
            options(ResumeMode::Never),
        )
        .await
        .unwrap();
    drop(first);
    invocations.0.lock().unwrap().clear();

    let current = if removed {
        Arc::new(Vec::new())
    } else {
        installed_tools(new_id, &invocations)
    };
    if !removed {
        assert_ne!(old_name, current[0].name());
    }
    let visible = Arc::new(Mutex::new(Vec::new()));
    let mut restored = session_builder_for_runtime(
        Arc::new(GuardedDriver {
            tools: current.clone(),
            visible: visible.clone(),
        }),
        pioneer,
    )
    .codec(Arc::new(OpenHumanTranscriptCodec))
    .tool_snapshot(snapshot(&current))
    .session(locator, conversation, metadata())
    .build()
    .unwrap();
    let resume = restored
        .resume(&options(ResumeMode::Session))
        .await
        .unwrap();
    assert!(resume.loaded);
    assert_eq!(
        resume.history, prior.history,
        "reinstall must preserve the exact conversation"
    );
    assert!(
        restored
            .recorded_tools()
            .unwrap()
            .specs()
            .iter()
            .any(|tool| tool.name == old_name),
        "historical declarations remain durable records, not execution authority"
    );
    let result = restored
        .turn(
            SessionTurnRequest::new(tinyinference_llm::message::Message::user("Continue")),
            options(ResumeMode::Session),
        )
        .await;
    if !pioneer {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-executable tools"));
        assert!(
            invocations.0.lock().unwrap().is_empty(),
            "generic behavior and executable guard stay intact"
        );
        return;
    }
    let result =
        result.expect("current authorized surface must replace stale runtime declarations");
    assert_eq!(
        &result.history[..prior.history.len()],
        prior.history.as_slice()
    );
    assert!(!visible.lock().unwrap().contains(&old_name));
    if removed {
        assert!(
            visible.lock().unwrap().is_empty(),
            "a removed or disabled server is not restored from history"
        );
        assert!(invocations.0.lock().unwrap().is_empty());
    } else {
        assert_eq!(*visible.lock().unwrap(), vec![current[0].name().to_owned()]);
        assert_eq!(*invocations.0.lock().unwrap(), vec![new_id.to_owned()]);
    }
}

#[tokio::test]
async fn pioneer_runtime_reinstalled_mcp_tools_resume_without_losing_history() {
    restored_session(true, false).await;
}

#[tokio::test]
async fn pioneer_runtime_removed_mcp_server_is_not_reauthorized_by_history() {
    restored_session(true, true).await;
}

#[tokio::test]
async fn pioneer_runtime_generic_resume_retains_declarations_and_executable_guard() {
    restored_session(false, false).await;
}
