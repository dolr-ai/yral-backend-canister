# Install candid-extractor (official DFINITY tool for generating .did files from wasm)
if ! command -v candid-extractor &>/dev/null; then
  cargo install candid-extractor
fi

# Install dfx non-interactively
if ! command -v dfx &>/dev/null; then
  export DFXVM_INIT_YES=true
  DFX_VERSION=0.31.0 sh -c "$(curl -fsSL https://internetcomputer.org/install.sh)"
fi

# Install pocket-ic server (idempotent)
POCKET_IC_VERSION="7.0.0"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
POCKET_IC_BIN="$REPO_ROOT/pocket-ic"

if [ ! -f "$POCKET_IC_BIN" ]; then
  TMP="$(mktemp)"
  curl -fsSL "https://github.com/dfinity/pocketic/releases/download/${POCKET_IC_VERSION}/pocket-ic-x86_64-darwin.gz" -o "$TMP"
  gunzip -c "$TMP" > "$POCKET_IC_BIN"
  rm "$TMP"
  chmod +x "$POCKET_IC_BIN"
fi
