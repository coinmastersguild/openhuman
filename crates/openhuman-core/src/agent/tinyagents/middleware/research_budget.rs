//! Bound direct web research so a run that keeps finding more leads still answers.

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyagents_harness::error::Result as TaResult;
use tinyagents_harness::middleware::{
    Middleware, MiddlewareToolOutcome, ToolHandler, ToolInvocationIdentity, ToolMiddleware,
};
use tinyinference_llm::message::Message;
use tinyinference_llm::model::{ModelRequest, ToolChoice};
use tinyinference_llm::tool::ToolCall;
use tinytools::ToolResult;

use crate::agent::tinyagents::host::OpenHumanRunContext;

/// Direct web reads allowed before the next model call must conclude the turn.
/// This bounds the master agent's exploratory search while leaving room to read
/// primary sources; broad research belongs in one deep `web_answer_tool` call or
/// a batched `web_contents_tool` read, each of which counts once.
pub(super) const DIRECT_WEB_READ_LIMIT: usize = 8;

const RESEARCH_CLOSE_INSTRUCTION: &str = "The direct web research budget for this turn is exhausted. Answer the user's latest request now using the results already available. State any remaining uncertainty. Do not search again, repeat a page fetch, or merely describe what you plan to read.";

#[derive(Default)]
pub(crate) struct ResearchBudgetMiddleware {
    completed_reads: AtomicUsize,
    pioneer: bool,
}

impl ResearchBudgetMiddleware {
    pub(crate) fn new() -> Self {
        Self::for_runtime(pioneer_runtime())
    }

    pub(crate) fn for_runtime(pioneer: bool) -> Self {
        Self {
            pioneer,
            ..Self::default()
        }
    }
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

#[async_trait]
impl ToolMiddleware<(), OpenHumanRunContext> for ResearchBudgetMiddleware {
    fn name(&self) -> &str {
        "research_budget"
    }

    async fn wrap_tool(
        &self,
        ctx: &mut RunContext<OpenHumanRunContext>,
        state: &(),
        call: ToolCall,
        next: ToolHandler<'_, (), OpenHumanRunContext>,
    ) -> TaResult<MiddlewareToolOutcome> {
        next.run(ctx, state, call).await
    }
}

#[cfg(test)]
#[path = "research_budget_pioneer_tests.rs"]
mod pioneer_tests;

#[async_trait]
impl Middleware<(), OpenHumanRunContext> for ResearchBudgetMiddleware {
    fn name(&self) -> &str {
        "research_budget"
    }

    async fn after_tool(
        &self,
        _ctx: &mut RunContext<OpenHumanRunContext>,
        _state: &(),
        invocation: &ToolInvocationIdentity,
        _result: &mut ToolResult,
    ) -> TaResult<()> {
        if matches!(
            invocation.tool_name(),
            "web_search_tool" | "web_answer_tool" | "web_contents_tool" | "web_fetch"
        ) {
            self.completed_reads.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn before_model(
        &self,
        _ctx: &mut RunContext<OpenHumanRunContext>,
        _state: &(),
        request: &mut ModelRequest,
    ) -> TaResult<()> {
        if self.completed_reads.load(Ordering::Relaxed) < DIRECT_WEB_READ_LIMIT {
            return Ok(());
        }
        tracing::info!(
            web_reads = self.completed_reads.load(Ordering::Relaxed),
            "[tinyagents::mw] direct web research budget reached; concluding turn"
        );
        request.tools.clear();
        request.tool_choice = ToolChoice::None;
        request
            .messages
            .push(Message::user(RESEARCH_CLOSE_INSTRUCTION.to_string()));
        Ok(())
    }
}
