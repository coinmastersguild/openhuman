use super::*;
use std::sync::Arc;
use tinyagents_harness::context::RunConfig;
use tinyagents_harness::middleware::{BoxToolFuture, MiddlewareStack, ToolBaseCall};
use tinyinference_llm::tool::ToolSchema;

struct CountingExecutor(AtomicUsize);

impl ToolBaseCall<(), OpenHumanRunContext> for CountingExecutor {
    fn call<'a>(
        &'a self,
        _ctx: &'a mut RunContext<OpenHumanRunContext>,
        _state: &'a (),
        _call: ToolCall,
    ) -> BoxToolFuture<'a> {
        Box::pin(async move {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(ToolResult::success("executed"))
        })
    }
}

fn context() -> RunContext<OpenHumanRunContext> {
    RunContext::new(
        RunConfig::new("research-budget"),
        OpenHumanRunContext::new(),
    )
}

fn call(name: &str) -> ToolCall {
    ToolCall {
        id: name.into(),
        name: name.into(),
        arguments: serde_json::json!({}),
        invalid: None,
    }
}

#[tokio::test]
async fn pioneer_runtime_eighth_read_keeps_authoring_and_ninth_cannot_execute() {
    let middleware = Arc::new(ResearchBudgetMiddleware::for_runtime(true));
    let mut stack = MiddlewareStack::default();
    stack.push_tool_middleware(middleware.clone());
    let executor = CountingExecutor(AtomicUsize::new(0));
    let mut ctx = context();
    for _ in 0..8 {
        let outcome = stack
            .run_wrapped_tool(&mut ctx, &(), call("web_fetch"), &executor)
            .await
            .unwrap()
            .into_result();
        assert!(!outcome.is_error);
    }
    let ninth = stack
        .run_wrapped_tool(&mut ctx, &(), call("web_fetch"), &executor)
        .await
        .unwrap()
        .into_result();
    assert!(
        ninth.is_error,
        "the ninth direct web read must stop before execution"
    );
    assert_eq!(executor.0.load(Ordering::SeqCst), 8);

    let mut request = ModelRequest::new(vec![Message::user(
        "Research references then author an animation",
    )]);
    request.tools = vec![
        ToolSchema::new("web_fetch", "web", serde_json::json!({})),
        ToolSchema::new("shell", "local authoring", serde_json::json!({})),
    ];
    middleware
        .before_model(&mut ctx, &(), &mut request)
        .await
        .unwrap();
    assert_eq!(
        request
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>(),
        vec!["shell"]
    );
    assert_ne!(request.tool_choice, ToolChoice::None);
    let authored = stack
        .run_wrapped_tool(&mut ctx, &(), call("shell"), &executor)
        .await
        .unwrap()
        .into_result();
    assert!(!authored.is_error);
    assert_eq!(executor.0.load(Ordering::SeqCst), 9);
}

#[tokio::test]
async fn pioneer_runtime_generic_eight_reads_still_conclude_all_tools() {
    let middleware = ResearchBudgetMiddleware::for_runtime(false);
    let mut ctx = context();
    for _ in 0..8 {
        middleware
            .after_tool(
                &mut ctx,
                &(),
                &ToolInvocationIdentity::new("fetch", "web_fetch"),
                &mut ToolResult::success("page"),
            )
            .await
            .unwrap();
    }
    let mut request = ModelRequest::new(vec![Message::user("research")]);
    request.tools = vec![ToolSchema::new(
        "shell",
        "local authoring",
        serde_json::json!({}),
    )];
    middleware
        .before_model(&mut ctx, &(), &mut request)
        .await
        .unwrap();
    assert!(request.tools.is_empty());
    assert_eq!(request.tool_choice, ToolChoice::None);
}
