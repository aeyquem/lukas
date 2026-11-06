# Repository guidance

This repository is being rewritten from a clean implementation state.

## Current direction

- Build a Rust backend and a web application.
- Make self-hosting the initial deployment model; keep a later SaaS option
  possible.
- Give the backend and frontend separate Dockerfiles for independent deployment;
  production must not require Compose. Use Docker Compose with the local Docker
  Engine for development. Use PostgreSQL for the first slice and the initial
  self-hosted deployment. SQLite is deferred as a possible later local desktop
  option; PostgreSQL provisioning and operating details remain open.
- The first self-hosted instance is owner-only; the owner may manage personal
  or household finances. Separate member identities are later scope.
- Provide a CLI for both human and agent use to query and add data.
- MCP is deprecated for now.

The agreed context and known domain constraints are in
`desc/00-rewrite-context.md`. Read it before making product or architecture
decisions. The owner delegated backend technical decisions with readability for
a Rust beginner as the primary constraint; use `desc/10-backend-architecture.md`
for the selected backend architecture and remaining open items. Read
`desc/09-technical-stack-proposal.md` for the broader technical proposal.
Direct owner discussions and accepted decisions in the rewrite context,
acceptance criteria and orchestrator take precedence. Other recommendations
remain proposals unless selected or accepted. The remaining files under `docs/` describe
the previous PoC and are obsolete for current rewrite work. Do not use them as
product or implementation requirements unless the user explicitly asks. Their
Tauri/.NET/Vue/pnpm guidance is not current.

Keep financial provenance, reviewability, auditable corrections, decimal-safe
money, real transaction dates, period context and explainable projections in
view when shaping the new design. The rewrite context lists unresolved choices;
do not present those as already decided.

## Aentic work

All processes should have a writer and a reviewer.
Thehe writer has the full context and proposes the changes
The reviewer take the changes and finds reasons on why they fail and pass them to the writer.
The cycle shold keep going until the writer passes the audit of the reviewer.

each agent has to be specialized in their area of expertice

## Interactive privileged commands

When a necessary local command requires `sudo` or access the current sandbox
cannot get (such as the local Docker Engine), run `tmux attach -t Interactive`
and execute it there so the owner can enter a password directly if prompted.
Do not put passwords in commands, scripts, environment variables, or chat. Send
only the needed command to that session, leave password entry to the owner, then
inspect the command output before continuing. Prefer the existing session over
starting a separate privileged shell.
