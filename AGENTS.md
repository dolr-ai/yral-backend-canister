# AGENTS.md

## Purpose

This is the living agent guide for the `yral-backend-canister` repository. It captures current repository-specific conventions, workflow patterns, and instructions for an agent operating here.

This file is not a changelog. It should describe the active way this repository is expected to be used and updated. When repository-wide conventions change, update this file immediately so future agents can rely on it.

## Repository Overview

- This repo contains backend canisters for the HotOrNot/yral Internet Computer project.
- The root uses `dfx` and canister-based Rust crates under `src/canister/`.
- Local development and CI are driven from repository scripts, not ad hoc commands.
- `canister_ids.json` and `sns_canister_ids.json` are authoritative canister ID manifests for this repo.

## Key Patterns

- Prefer the repository's canonical scripts for commands instead of inventing new tooling.
  - `bash scripts/run_canister_test_suite.sh` is the canonical local/CI test entrypoint.
  - `bash scripts/canister_snapshot.sh` is used for snapshot actions: `ACTION=take_snapshot`, `ACTION=list_snapshots`, and `ACTION=load_snapshot`.
  - `bash scripts/release_and_submit_proposals.sh` is the canonical release/proposal entrypoint.

- Use the root `README.md` as the primary source of human-facing repository workflow documentation.
- For canister compile/runtime details, rely on `dfx.json` plus the code in `src/canister/*`.
- Do not edit or rely on build artifacts under `target/` or `wasms/` as source files.

## Testing and Upgrade Workflow

- The repo emphasizes reproducible local runs that match CI.
- The `scripts/ci` scripts are source-of-truth for CI and local execution.
- `ic_repl_tests/` contains local upgrade and integration-style test scenarios.
- Upgrade testing is currently manual: start `dfx`, check out previous tag, run the suite, switch to `main`, run the suite again, and verify canister version values.

## Deployment Pattern

- Mainnet deployment is handled via GitHub Actions and proposal submission.
- The sequence is:
  1. Merge PR to `main`.
  2. Tag a semver release.
  3. Push the tag.
  4. Let GitHub Actions trigger the upgrade proposal flow.
- Validate module hashes using `dfx canister info <canister-id> --network=ic` and cross-check GH action deployment output.

## Agent Behavior Rules

- Always check `README.md` and `scripts/*` first for the current workflow.
- Avoid making arbitrary changes to canister deployment or upgrade behavior without explicit evidence from repo docs or tests.
- If a new process is introduced, document it here and keep the language prescriptive.
- Keep agent edits minimal when updating workflows: update the official script or docs, then update `AGENTS.md`.

## When to Update This File

Update `AGENTS.md` whenever any of the following change:

- The canonical test or deployment script changes.
- The repository adds or removes a major canister or canister manifest file.
- The release/tagging/proposal process changes.
- The local reproducibility workflow changes.
- A new high-level engineering convention appears that future agents must know.

When updating, keep it terse and current. Remove obsolete patterns immediately.

## Self-Update Instructions

This file is the authoritative agent reference for this repository. If you are an agent making changes to repo-wide conventions:

- Change `AGENTS.md` as part of the same PR.
- Summarize the changed convention in a short new paragraph or bullet.
- Keep the content focused on the active repository state.
- Do not preserve old workflows as permanent content.

If a section of this repo becomes obsolete, delete it from this file instead of retaining it as historical context.

## HANDOFF.md Handling

- There is currently no `HANDOFF.md` in this repository.
- If a future agent sees a `HANDOFF.md` file:
  - Read it fully and absorb the exact resume state and next steps.
  - Migrate any relevant instructions into `AGENTS.md` if they represent ongoing repository conventions.
  - Remove `HANDOFF.md` after its context has been absorbed and the handoff is complete.

## Notes for Future Agents

- This repo is strongly centered on Internet Computer canisters and DFX tooling.
- Root-level scripts are the main integration points for developer workflows.
- If you need to experiment, prefer the documented `bash scripts/...` flows rather than creating new command conventions.
- Keep the living nature of this document in mind: it should reflect how this repository is actually used today.
