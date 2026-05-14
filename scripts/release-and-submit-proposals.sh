#!/usr/bin/env bash
# Build canister wasms and submit SNS upgrade proposals directly from a local machine.
# Must be run from the repo root.
#
# Usage:
#   VERSION=v1.1.0 bash scripts/release-and-submit-proposals.sh
#   VERSION=v1.1.0 CHANGE_SUMMARY="Strip individual_user_template to hello_world" bash scripts/release-and-submit-proposals.sh
#
# Prerequisites:
#   - actions_identity.pem: paste your SNS proposal submitter PEM key into this file (gitignored)
#   - dfx installed: bash scripts/install-dependencies.sh
set -euo pipefail

VERSION="${VERSION:-}"
NEURON_ID="4de673e9cd7a1339afea6523a5f227d25e9d739ff52635ac86dbdb0447ae106a"
IDENTITY_FILE="actions_identity.pem"

if [[ -z "$VERSION" ]]; then
  echo "Error: VERSION is required."
  echo "Usage: VERSION=v1.1.0 bash scripts/release-and-submit-proposals.sh"
  exit 1
fi

if [[ ! -f "$IDENTITY_FILE" ]] || ! grep -q "BEGIN" "$IDENTITY_FILE" 2>/dev/null; then
  echo "Error: $IDENTITY_FILE not found or does not contain a PEM key."
  echo "Paste your SNS proposal submitter PEM key into $IDENTITY_FILE and re-run."
  exit 1
fi

CHANGE_SUMMARY="${CHANGE_SUMMARY:-Upgrade canister fleet to ${VERSION}}"

# Import the proposal submitter identity into dfx
dfx identity import --storage-mode=plaintext actions "$IDENTITY_FILE" --force
dfx identity use actions

# Build canisters for mainnet and print module hashes
for canister in platform_orchestrator individual_user_template user_index; do
  echo "==> Building $canister for mainnet..."
  dfx build "$canister" --network=ic
  hash=$(sha256sum < ".dfx/ic/canisters/${canister}/${canister}.wasm.gz")
  echo "    ${canister} module hash: ${hash}"
done

# Download quill (macOS) if not present
if [[ ! -x ./quill ]]; then
  echo "==> Downloading quill..."
  curl -fsSL -o quill \
    https://github.com/dfinity/quill/releases/download/v0.4.2/quill-macos-x86_64
  chmod +x quill
fi

# Download ic-repl (macOS) if not present
if [[ ! -x ./ic-repl ]]; then
  echo "==> Downloading ic-repl..."
  ICREPL_VERSION=$(curl -s https://api.github.com/repos/dfinity/ic-repl/releases/latest \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])")
  curl -fsSL -o ic-repl \
    "https://github.com/dfinity/ic-repl/releases/download/${ICREPL_VERSION}/ic-repl-macos"
  chmod +x ic-repl
fi

# Submit SNS upgrade proposal for platform_orchestrator (direct canister upgrade)
canister_name="platform_orchestrator"
canister_id="$(dfx canister id "$canister_name" --network=ic)"
mkdir -p "proposals/${canister_name}"

echo "==> Submitting SNS upgrade proposal for ${canister_name}..."
./quill sns \
  --canister-ids-file ./sns_canister_ids.json \
  --pem-file "$IDENTITY_FILE" \
  make-upgrade-canister-proposal \
  --title "Upgrade ${canister_name}" \
  --summary "# Upgrade ${canister_name}\n\n${CHANGE_SUMMARY}" \
  --url 'https://yral.com' \
  --target-canister-id "$canister_id" \
  --wasm-path ".dfx/ic/canisters/${canister_name}/${canister_name}.wasm.gz" \
  --canister-upgrade-arg "(record {version=\"${VERSION}\"})" \
  "$NEURON_ID" > "proposals/${canister_name}/upgrade.json"

./quill send "proposals/${canister_name}/upgrade.json" --yes

# Submit SNS generic function proposals for user_index and individual_user_template.
# These go through platform_orchestrator's UpgradeSubnetCanisters generic function (id 4002),
# which distributes the wasm to the entire canister fleet.
for canister_name in user_index individual_user_template; do
  echo "==> Submitting SNS proposal for ${canister_name}..."
  CANISTER_NAME="$canister_name" \
  CHANGE_SUMMARY="$CHANGE_SUMMARY" \
  VERSION="$VERSION" \
    ./ic-repl scripts/upgrade_ic_repl.sh -r ic
done

rm -rf proposals
echo ""
echo "Done. All proposals submitted for version ${VERSION}."
