# Pioneer fork build and contribution boundary

The public runtime fork is coinmastersguild/openhuman, default branch pioneer.
The source baseline is upstream v0.64.10 (2eb8a33049915cc5e4b6c10c5c9f2ab99862e0ea).
Only public upstream source is present. Private control-plane code, tenant data
and credentials must never be added. All PRs and improvements target this fork;
upstream is fetch-only and must never receive PRs, issues, messages or pushes.
Owner review and passing CI are required before a runtime release.

## Prepared native behavior

PIONEER_LOCAL_RUNTIME selects fixed Pioneer Analytic and GLM gateway routes with
the tenant MODEL_API_KEY. The native computer planner, rescue, output and
analytics decisions have no hosted fallback. Tool search uses local lexical
retrieval followed by analytics. Chrome opens visibly on DISPLAY and routes
public sites through the tenant egress proxy; loopback pages bypass that proxy.
The patched TinyComputer contract is 2.9 and is compiled from pinned public
sources. The core embeds its exact module SHA256 and fails closed if the module
is absent or differs. Linux artifacts are built with the minimal hosted-agent
feature set; voice, media capture and web3 features are outside this build.

## Verification and remaining live acceptance

Verified on the build host: 591 native module/contract/engine unit and API
tests, 35 native doctests, 52 decisions-client tests plus API and doctest,
and 35 tinyjev-client tests plus API and doctest. All passed. All three host routing/status regressions and contribution guards passed.
Inherited upstream release publishers are explicitly disabled in this fork,
with a regression guard. The clean pinned Linux release rebuild completes
before image use.
Before integration, rerun host configuration tests and the Linux build script.
The build receipt records source pins, dependency patch hashes and artifacts.
No script installs an image, restarts services or deploys a runtime.

A native browser tool also requires trusted browser.enabled configuration;
module readiness alone does not prove the agent can invoke it. Live acceptance
must show a native decision using local Clef, a GLM planner request, no public
inference egress, a headed browser action and a resulting workspace artifact.
Continuous Studio desktop streaming must remain visible through those actions.
Vision support of the deployed model and Clef image decisions are unverified;
there is no external fallback if either is unsupported. Blender needs its own
successful application/tool test. Source tests and a screenshot alone cannot
claim these live integration gates passed.
