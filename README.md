# HotOrNot Backend Canisters

## Reproducible local runs (same as CI)

The repository uses scripts in `scripts/ci` as the source of truth for both GitHub Actions and local runs.

Run these from the devcontainer:

```sh
bash scripts/run_canister_test_suite.sh
```

```sh
ACTION=take_snapshot CANISTER_ID=<canister-id> bash scripts/canister_snapshot.sh
```

```sh
ACTION=list_snapshots CANISTER_ID=<canister-id> bash scripts/canister_snapshot.sh
```

```sh
ACTION=load_snapshot CANISTER_ID=<canister-id> SNAPSHOT_ID=<snapshot-id> bash scripts/canister_snapshot.sh
```

You can also run the same commands from the VS Code Task UI via:
- `CI: Run canister test suite`
- `CI: Snapshot take`
- `CI: Snapshot list`
- `CI: Snapshot load`
- `CI: Release and submit proposals`
- `CI: Add PR to project`
- `CI: Publish image`


## Testing Upgrades Locally
Before pushing changes to canisters on mainnet we make sure the upgrade will run successfully. Right now there is no automated way to do it. To test it locally follow the steps.

### Step 0
Run clean dfx 

```sh
dfx start --clean --background
```

### Step 1
Checkout to last git tag
```sh
git checkout vx.y.z
```

### Step 2
Run the CI-equivalent test suite script on the old tag
```sh
bash scripts/run_canister_test_suite.sh
```

### Step 3
Checkout to the head of main branch that we need to deploy
```sh
git checkout main
```

### Step 4
Run the same CI-equivalent suite on `main`
```sh
bash scripts/run_canister_test_suite.sh
```

### Step 5
Check version for any of the individual canisters deployed locally. If version is greater than `v1.0.0` then the upgrades were successful.

```sh
dfx canister call <individual-canister-id> get_version
```

## Mainnet Deployment

### Mainnet Deployment Checks
These checks are important and should be strictly performed before raising any Pull request and ensure everything passes.

- checkout to the latest tag before the current build.
- deploy the canisters locally.
- Run the ic repl tests: `ic_repl_tests/all_tests.sh` (this would create some users locally and will add some posts for testing.)
- checkout to your branch
- run the upgrade process described above without skipping the tests.
- check if all the user canisters upgrade successfully and the posts are retained which were added by repl tests.


### Mainnet Deployment

The process of deploying to the mainnet is as follows:
- merge the Pull requests to the main branch
- create a semver tag for the release and push it.
- A github action would be triggered and raise the necessary proposals to upgrade the canisters

## Verifying builds

To get the hash for canisters:

- Get the canister IDs from [`canister_ids.json`](https://github.com/go-bazzinga/hot-or-not-backend-canister/blob/main/canister_ids.json).
- Get hash using the DFX SDK by running: `dfx canister info <canister-id> --network=ic`.

- The output of the above command should contain `Module hash` followed up with the hash value. Example output:

  ```
  $ > dfx canister info vyatz-hqaaa-aaaam-qauea-cai --network=ic

  Controllers: 7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae
  wwyo5-vrahh-jwa74-3m6kj-jqbia-jbebm-7vtyd-uvqem-wk3zw-djpci-vqe
  Module hash: 0x98863747bb8b1366ae5e3c5721bfe08ce6b7480fe4c3864d4fec3d9827255480
  ```

To get the hash for canister deployment:

- Go to [Github actions deployment runs](https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/workflows/webclient-deploy.yml)
- Open the latest successful run. ([Click to see an example run](https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/runs/4810296657))
- Go to any of the `Deploy all canisters` jobs. ([Click to see an example job](https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/runs/4900015913/jobs/8750374252))
- Open one of the `Deploy <canister_name> canister` steps. You should find the `Module hash` in this step. This value should match the value you got locally. ([Click to see an example step](https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/runs/4900015913/jobs/8750374252#step:8:16))

To check the status of the deployment

- check if the platform orchestrator performed the step to upgrade subnet canister with appropriate version: [Platform Orchestrator function](https://dashboard.internetcomputer.org/canister/74zq4-iqaaa-aaaam-ab53a-cai#get_subnet_last_upgrade_status)
- check the status of upgrade for individual canisters in subnet orchestrators and verify the version. Example for one of the subnet orchesrator: [Subnet Orchestrator function](https://dashboard.internetcomputer.org/canister/rimrc-piaaa-aaaao-aaljq-cai#get_index_details_last_upgrade_status)

---