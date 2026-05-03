# HotOrNot Backend Canisters

## Running locally

Run via VS Code Task UI (`Ctrl+Shift+P` → `Tasks: Run Task`):

- `CI: Run canister test suite`
- `CI: Snapshot take`
- `CI: Snapshot list`
- `CI: Snapshot load`
- `CI: Release and submit proposals`

## Verifying a deployment

After a tag is pushed and the GitHub Actions release workflow completes:

- [Deployment runs](https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/workflows/webclient-deploy.yml) — open the run and expand a `Deploy <canister_name> canister` step to find the `Module hash`.
- Cross-check on-chain: `dfx canister info <canister-id> --network=ic` — the `Module hash` field must match.
- Canister IDs: [`canister_ids.json`](https://github.com/go-bazzinga/hot-or-not-backend-canister/blob/main/canister_ids.json)

Monitor upgrade propagation:

- [Platform Orchestrator](https://dashboard.internetcomputer.org/canister/74zq4-iqaaa-aaaam-ab53a-cai#get_subnet_last_upgrade_status)
- [Subnet Orchestrator (example)](https://dashboard.internetcomputer.org/canister/rimrc-piaaa-aaaao-aaljq-cai#get_index_details_last_upgrade_status)

---