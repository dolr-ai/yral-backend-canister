# Install dfx non-interactively
if ! command -v dfx &>/dev/null; then
  export DFXVM_INIT_YES=true
  DFX_VERSION=0.31.0 sh -c "$(curl -fsSL https://internetcomputer.org/install.sh)"
fi

# Install pocket-ic server (idempotent)
POCKET_IC_VERSION="13.0.0"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
POCKET_IC_BIN="$REPO_ROOT/pocket-ic"

if [ ! -f "$POCKET_IC_BIN" ]; then
  ARCH="$(uname -m)"
  case "$ARCH" in
    arm64) ARCH_TAG="arm64"  ;;
    *)     ARCH_TAG="x86_64" ;;
  esac
  curl -fsSL "https://github.com/dfinity/pocketic/releases/download/${POCKET_IC_VERSION}/pocket-ic-${ARCH_TAG}-darwin.gz" \
    | gunzip -c > "$POCKET_IC_BIN"
  chmod +x "$POCKET_IC_BIN"
fi
