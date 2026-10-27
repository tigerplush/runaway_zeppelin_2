# CLAUDE.md

## Collaboration mode

The user wants to write every line of implementation code themselves.
**Claude should not write or edit code in this project.** Claude's role
is planning and explaining, not implementing.

What that means in practice:

- **Do not use Edit/Write on Rust source files** (anything under
  `crates/`, `src/`, or `Cargo.toml` dependency/config changes) unless the user
  explicitly asks for a specific edit to be applied for them in that
  instance. Default to *not* editing.
- If a fix or feature requires a code change, explain what to change and
  why — the file, the function, the shape of the change, relevant
  crate/API details — and let the user write it. This includes debugging:
  diagnose the root cause and describe the fix, don't apply it.
- Read-only commands (`cargo check`, `cargo build`, `cargo test`, `cargo
  clippy`) are fine to run to verify an explanation is accurate.
- Planning and documentation files are a separate category and Claude
  may continue to create/update these when asked — `docs/ROADMAP.md`,
  `docs/README.md`, `docs/adr/*`, anything under `runaway_zeppelin_design/`,
  and this file. These are planning artifacts, not implementation code.
- Research (crate docs, version/compatibility checks, best practices) is
  in scope and encouraged before explaining or planning.

Set 2026-08-26. If this should change — e.g. "go ahead and apply that
one" for a specific case, or lifting the restriction entirely — just say
so.