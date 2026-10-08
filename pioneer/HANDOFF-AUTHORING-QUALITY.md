# Pioneer authoring quality

Base: `5bd3b5b73db158dbba2e01c8c9a5e2dc7165a599` on this fork's `pioneer` branch.
Contributions and releases stay in `coinmastersguild/openhuman`; never upstream.

The acceptance regressions cover continuing local authoring after eight direct
web reads while refusing the ninth read, capability-derived visual authoring
guidance, and waiting for Pioneer image critique within the current turn.
Generic research conclusion and explicit background delegation stay unchanged.
No tool authority, provider route, deadline or prepaid budget is broadened.

Tests-first `269d64d` RED: 23 existing/compatibility tests passed and four new
behavioral regressions failed at the intended boundaries. `fafef65` adds the
parallel-read regression before implementation. The candidate implements only
the protected Pioneer policies above. Linux GREEN and independent source review
have passed: 28 Pioneer host checks and 24 fork governance/provenance checks.
Existing generic research and delegation checks passed (2 and 9 respectively).
The two legacy prompt assertions now distinguish generic provider routes from
Pioneer's current-tool-only routes. Independent source review approves the
candidate; a core-only native build is in progress.
Real integration and visual quality acceptance remain separate release gates.

The eight-read admission bound applies only to `web_search_tool`,
`web_answer_tool`, `web_contents_tool` and `web_fetch`. MCP, browser and curl
operations retain their own tool admission, size/time limits and the existing
overall turn budget; this policy is not a total network-read quota.
