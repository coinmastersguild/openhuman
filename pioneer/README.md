# Pioneer OpenHuman runtime

This public fork starts from upstream `v0.64.10`, commit
`2eb8a33049915cc5e4b6c10c5c9f2ab99862e0ea`. The default branch is `pioneer`.
All contributions, agent-authored improvements and pull requests target
**coinmastersguild/openhuman:pioneer only**. Never open upstream pull requests,
issues, or messages, or push to upstream. Upstream remains a read-only source.
Run `node scripts/pioneer/guard.mjs --configure` after cloning to make `origin`
the only push remote and to set GitHub CLI's default to this fork.

Agent self-improvement means authoring and testing a branch in this repository,
then proposing it for owner review. It does not grant unattended runtime releases,
merge rights, organization-wide access or access to private Pioneer repositories.
A runtime release requires owner approval, passing CI and a pinned build receipt.
No tenant credentials, private control-plane code or tenant data belong here.

Hosted project grants remain read-only and limited to one personal project
repository. Autonomous runtime PR submission is not implemented. The planned
flow is a runtime-authored patch followed by an Alpha owner-approved write
executor, targeting only **coinmastersguild/openhuman**. It must never target
private Pioneer repositories, other organization repositories or upstream.
The tenant receives no write credential.

## Local inference

The trusted host sets `PIONEER_LOCAL_RUNTIME=1` and supplies the tenant's scoped
`MODEL_API_KEY`. Computer decisions and tool search call Pioneer Analytic at
`http://10.88.0.1:12500/v1/systemone`. Planner, rescue and output models call GLM
at `http://10.88.0.1:12500/v1`, using `glm-5.3-flash-local`. These are fixed
container-to-host gateway routes. The gateway alone calls host loopback services;
container `127.0.0.1` refers to the container itself. Requests use the existing
per-tenant metering and budget rather than an Alpha billing round trip.

No local configuration sends those calls through Alpha, Cloudflare's public
tunnel, TinyHumans or OpenRouter. Missing scoped identity fails closed. Tool
search uses a local BM25 shortlist followed by a Pioneer Analytic choice; it
needs no hosted embedding provider. The existing harness BM25 fallback remains
available when analytics is unavailable.

If this runtime's local gateway returns `402` with `token budget exhausted`,
chat asks the owner to top up that agent in Pioneer Studio. This is the agent's
prepaid inference budget, not a TinyHumans cloud plan or a reason to change its
model/API key. The same failed turn is not automatically retried. Generic local
packages and unrelated provider failures keep their existing guidance.

`pioneer/tinycomputer-local.patch` modifies only the pinned public TinyComputer
source at `16446e009c3c2158e5a1bfec24b041d547776e30`: it adds exact local provider
routes, keeps unrelated destinations rejected, and makes the browser visible on
the tenant's desktop. The Pioneer contract is 2.9 so a mismatched older module
is rejected. Chrome uses the fixed tenant egress proxy for public websites, with
loopback bypass for pages served inside that same tenant. `pioneer/tinyinference-decisions-local.patch` and `pioneer/tinyjevclient-local.patch`
add the exact same origin exception to the underlying clients; other plaintext
hosts, other ports, URL credentials, queries and fragments stay refused. The
clients remain pinned to their public revisions. Copyright and licenses remain
intact; these are modified Pioneer builds, not upstream releases.

## Build and desktop acceptance

On Linux, initialize recursive submodules, install the pinned Rust toolchain,
then run `scripts/pioneer/build-linux.sh`. The script builds the native module
and main binary from this fork, writes hashes and provenance to ignored
`pioneer/artifacts/`, and performs no service restart. The trusted image installs
`openhuman-core`, the module and `modules.toml`; the module goes at
`/opt/pioneer/lib/libtinycomputer.so`. The binary embeds its expected hash and
refuses an absent or different artifact instead of downloading an upstream one.
Pioneer mode skips generic module search paths and overrides, refuses an unknown
preloaded TinyComputer, and rejects other optional native modules until they
have an explicit trusted image pin. A build carrying the compiled module pin
forces this policy even if user environment values clear the local-mode switch.
MCP and shell tools remain available.

The tenant's full desktop must remain available through an owner-authorized
single-use Studio session and a continuous binary WebSocket framebuffer stream.
Acceptance requires a real browser rendering that desktop while the agent opens
a visible browser, acts on a test page, and creates a verifiable workspace
artifact. Blender and other desktop applications need the same visible tenant
display and a tested application/tool path. A chat transcript, screenshot alone,
or an untested VNC connection is not evidence of that flow.

## Local agent-template package

The local agent-template uses a separate generic core from this fork, built with
`scripts/pioneer/build-local-linux.sh pioneer-local-v0.1.0` on Linux amd64. Its
offline chat, MCP tools and scheduler use the user's MODEL_* settings. Native
modules and the hosted decision ranker are excluded from this package; it has no
Pioneer tenant compile pin and does not assume the Beast gateway exists locally.
It is not the trusted tenant build and makes no local full-desktop claim.

Only a reviewed fork release may publish the explicit core/license/receipt
archive. The template must pin both its fork release URL and archive SHA256.
The package builder requires every recursive dependency at its recorded public
gitlink, with only the committed patch overlays. It compares actual file
bytes to the public trees before and after patching, and rejects extra tracked,
untracked or ignored compiler inputs. Generated build outputs are not packaged.
Initial packages target Linux amd64; Apple Silicon uses Docker amd64 emulation.
Native arm64 packages will require their own tested build and receipt.


## Inline MCP image feedback

The pinned `tinymcp-images.patch` retains static PNG, JPEG and WebP tool images
as native image follow-ups for both configured tools and generic MCP calls.
Images never become megabytes of encoded text. The adapter preserves text and
image order, marks the attachments as untrusted tool output, and does not fetch
URLs or files. A still preview does not verify animation or a complete video.

Each result permits at most two images, two MiB decoded per image, four MiB
combined, 4096 pixels on either axis and 4,194,304 source pixels per image.
Canonical base64, MIME/signature agreement, static-image checks and a full
bounded decode must all pass. Invalid or animated attachments produce a fixed
error without echoing their bytes. A failed tool result never forwards images.
These changes are statically linked into `openhuman-core`; they require no MCP
plugin or change to the separately pinned TinyComputer module.

## Resuming Pioneer conversations after a restart

A Pioneer restart can reinstall MCP servers with new local identities and tool
names. Conversation history and recorded declarations stay intact, while the
next turn advertises the current authorized tool surface. Missing historical
declarations are not merged back into that surface. Current enabled servers
and their safe offered tools still govern MCP executor reconstruction; current
authoritative integration state still governs recorded integration actions.
The executable-tool check remains mandatory. Generic builds keep the original
recorded-declaration retention behavior.
