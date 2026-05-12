# AGENTS.md

## Purpose

This is the living agent guide for the `yral-backend-canister` repository. It captures current repository-specific conventions, workflow patterns, and instructions for an agent operating here.

This file is not a changelog. It should describe the active way this repository is expected to be used and updated. When repository-wide conventions change, update this file immediately so future agents can rely on it.

## Repository Overview

- This repo contains backend canisters for the HotOrNot/yral Internet Computer project.
- The root uses `dfx` and canister-based Rust crates under `src/canister/`.
- Local development and CI are driven from repository scripts, not ad hoc commands.
- `canister_ids.json` and `sns_canister_ids.json` are authoritative canister ID manifests for this repo.

## Canonical Scripts

All scripts live under `scripts/`. Use these — do not invent alternatives.

| Script | Purpose |
|--------|---------|
| `scripts/install-dependencies.sh` | Install dfx, pocket-ic, and candid-extractor (idempotent — safe to re-run) |
| `scripts/run-canister-test-suite.sh` | Full local/CI test suite |
| `scripts/generate-candid.sh` | Rebuild wasm(s) and regenerate `can.did` from the compiled output |
| `scripts/canister_snapshot.sh` | Canister snapshot operations (take / list / load) |
| `scripts/release-and-submit-proposals.sh` | Build release artifacts and submit SNS upgrade proposals |

### Running the test suite

```sh
bash scripts/install-dependencies.sh
bash scripts/run-canister-test-suite.sh
```

### Snapshot operations

```sh
ACTION=take_snapshot CANISTER_ID=<canister-id> bash scripts/canister_snapshot.sh
ACTION=list_snapshots CANISTER_ID=<canister-id> bash scripts/canister_snapshot.sh
ACTION=load_snapshot  CANISTER_ID=<canister-id> SNAPSHOT_ID=<snapshot-id> bash scripts/canister_snapshot.sh
```

## Testing Upgrades Locally

Before pushing canister changes to mainnet, verify the upgrade path:

1. `dfx start --clean --background`
2. `git checkout vx.y.z` — check out the last tag
3. `bash scripts/run-canister-test-suite.sh` — run suite on old tag
4. `git checkout main`
5. `bash scripts/run-canister-test-suite.sh` — run suite on new code
6. `dfx canister call <individual-canister-id> get_version` — confirm version is greater than `v1.0.0`

Also run `ic_repl_tests/all_tests.sh` to create test users and posts, then verify they are retained after upgrade.

## Mainnet Deployment

Pre-PR checklist:
- Run the full upgrade test above.
- Confirm all user canisters upgrade successfully and repl-test posts are retained.

Deployment sequence:
1. Merge PR to `main`.
2. Create and push a semver tag.
3. GitHub Actions triggers `create-release-on-tag-push.yml`, which builds artifacts and submits SNS upgrade proposals via `scripts/release-and-submit-proposals.sh`.

Verify after deployment:
- `dfx canister info <canister-id> --network=ic` — `Module hash` must match the hash printed during the Actions run.
- Canister IDs: `canister_ids.json`.

## CI Workflows

| Workflow | Trigger | Script |
|----------|---------|--------|
| `all-canisters-test-suite-on-any-push.yml` | Every push | `run-canister-test-suite.sh` |
| `create-release-on-tag-push.yml` | Tag push (`v*`) | `release-and-submit-proposals.sh` |
| `canister-snapshot.yml` | Manual dispatch | `canister_snapshot.sh` |

All workflows run on `macos-latest` and install dependencies via `scripts/install-dependencies.sh`.

## Agent Behavior Rules

- Always check `AGENTS.md` and `scripts/*` first for the current workflow.
- Avoid making arbitrary changes to canister deployment or upgrade behavior without explicit evidence from repo docs or tests.
- If a new process is introduced, document it here and keep the language prescriptive.
- Keep agent edits minimal when updating workflows: update the official script or docs, then update `AGENTS.md`.

## When to Update This File

Update `AGENTS.md` whenever any of the following change:

- The canonical test or deployment script changes (name or behavior).
- The repository adds or removes a major canister or canister manifest file.
- The release/tagging/proposal process changes.
- The local reproducibility workflow changes.
- A new high-level engineering convention appears that future agents must know.

When updating, keep it terse and current. Remove obsolete patterns immediately.

## Self-Update Instructions

This file is the authoritative agent reference for this repository. If you are an agent making changes to repo-wide conventions:

- Change `AGENTS.md` as part of the same commit.
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
