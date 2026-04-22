#!/usr/bin/env bash
set -euo pipefail

VERSION="${VERSION:-${GITHUB_REF_NAME:-}}"
NEURON_ID="${NEURON_ID:-}"
REPO="${GITHUB_REPOSITORY:-dolr-ai/yral-backend-canister}"

if [[ -z "$VERSION" ]]; then
  echo "VERSION or GITHUB_REF_NAME is required"
  exit 1
fi

if [[ -z "${HOT_OR_NOT_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY:-}" ]]; then
  echo "HOT_OR_NOT_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY is required"
  exit 1
fi

if [[ -z "$NEURON_ID" ]]; then
  echo "NEURON_ID is required"
  exit 1
fi

identity_file="actions_identity.pem"
printf "%s" "$HOT_OR_NOT_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY" > "$identity_file"

dfx identity import --storage-mode=plaintext actions "$identity_file" --force
dfx identity use actions

for canister in platform_orchestrator individual_user_template user_index
  do
  dfx build "$canister" --network=ic
  hash=$(sha256sum < ".dfx/ic/canisters/${canister}/${canister}.wasm.gz")
  echo "${canister} Module Hash: ${hash}"
done

cp .dfx/ic/canisters/platform_orchestrator/service.did platform_orchestrator.did
cp .dfx/ic/canisters/individual_user_template/service.did individual_user_template.did
cp .dfx/ic/canisters/user_index/service.did user_index.did

if [[ -n "${GH_TOKEN:-}" ]]; then
  previous_tag=$(git describe --tags --abbrev=0 "${VERSION}^" 2>/dev/null || true)
  if [[ -n "$previous_tag" ]]; then
    changelog=$(git log --pretty='- %s (%h)' "${previous_tag}..${VERSION}")
  else
    changelog=$(git log --pretty='- %s (%h)')
  fi

  gh release view "$VERSION" --repo "$REPO" >/dev/null 2>&1 || gh release create "$VERSION" \
    ./.dfx/ic/canisters/*/*.wasm.gz \
    ./*.did \
    --repo "$REPO" \
    --title "$VERSION" \
    --notes "${changelog:-Release $VERSION}"
fi

if [[ ! -x ./quill ]]; then
  curl -L -o quill https://github.com/dfinity/quill/releases/download/v0.4.2/quill-linux-x86_64-musl
  chmod +x quill
fi

if [[ -n "${CHANGE_SUMMARY:-}" ]]; then
  summary="$CHANGE_SUMMARY"
else
  summary="# Upgrade platform_orchestrator\n\nRelease ${VERSION}"
fi

canister_name="platform_orchestrator"
canister_id="$(dfx canister id "$canister_name" --network=ic)"
mkdir -p "proposals/${canister_name}"

./quill sns \
  --canister-ids-file ./sns_canister_ids.json \
  --pem-file "$identity_file" \
  make-upgrade-canister-proposal \
  --title "Upgrade ${canister_name} Canisters" \
  --summary "$summary" \
  --url 'https://yral.com' \
  --target-canister-id "$canister_id" \
  --wasm-path ".dfx/ic/canisters/${canister_name}/${canister_name}.wasm.gz" \
  --canister-upgrade-arg "(record {version=\"${VERSION}\"})" \
  "$NEURON_ID" > "proposals/${canister_name}/upgrade.json"

./quill send "proposals/${canister_name}/upgrade.json" --yes

for canister_name in user_index individual_user_template
  do
  CANISTER_NAME="$canister_name" CHANGE_SUMMARY="$summary" VERSION="$VERSION" ./ic-repl-linux64 ic-repl-upgrades-proposal/upgrade_ic_repl.sh -r ic
done

rm -rf proposals
