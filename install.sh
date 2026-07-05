#!/usr/bin/env bash
set -euo pipefail

G='\033[0;32m'; Y='\033[1;33m'; C='\033[0;36m'; NC='\033[0m'

echo -e "${G}══════════════════════════════════════════${NC}"
echo -e "${G}  WOPR TUI 2026 — Installer${NC}"
echo -e "${G}  Global Thermonuclear War Simulation${NC}"
echo -e "${G}══════════════════════════════════════════${NC}"
echo ""

OS="$(uname -s)"

# ── Rust ──
if ! command -v cargo &>/dev/null; then
    echo -e "${Y}Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo -e "${G}✓${NC} Rust $(rustc --version | cut -d' ' -f2)"

# ── macOS: Xcode CLI tools ──
if [ "$OS" = "Darwin" ] && ! xcode-select -p &>/dev/null; then
    echo -e "${C}Installing Xcode Command Line Tools...${NC}"
    xcode-select --install
    until xcode-select -p &>/dev/null; do sleep 5; done
fi

# ── Linux: cc + pkg-config + OpenSSL headers (reqwest needs them) ──
if [ "$OS" = "Linux" ]; then
    MISSING=""
    command -v cc         &>/dev/null || MISSING="$MISSING build-essential"
    command -v pkg-config &>/dev/null || MISSING="$MISSING pkg-config"
    [ -f /usr/include/openssl/ssl.h ] || [ -f /usr/include/x86_64-linux-gnu/openssl/ssl.h ] || MISSING="$MISSING libssl-dev"
    if [ -n "$MISSING" ]; then
        echo -e "${C}Installing system deps:${MISSING}${NC}"
        if   command -v apt-get &>/dev/null; then sudo apt-get update -qq && sudo apt-get install -y $MISSING
        elif command -v dnf     &>/dev/null; then sudo dnf install -y ${MISSING//build-essential/gcc} ${MISSING//libssl-dev/openssl-devel}
        elif command -v pacman  &>/dev/null; then sudo pacman -Sy --noconfirm ${MISSING//build-essential/base-devel} ${MISSING//libssl-dev/openssl}
        else echo "ERROR: Install manually:$MISSING"; exit 1; fi
    fi
fi

# ── Source: use local repo if present, otherwise clone to temp dir ──
CLEANUP=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}" 2>/dev/null)" && pwd 2>/dev/null)" || SCRIPT_DIR=""
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    SRC="$SCRIPT_DIR"
else
    SRC="$(mktemp -d)"
    CLEANUP="$SRC"
    echo -e "${C}Cloning WOPR TUI 2026...${NC}"
    git clone --depth 1 https://github.com/ankurCES/WOPR_TUI_2026.git "$SRC"
    SRC="$SRC/WOPR_TUI_2026"
fi

# ── Build + install ──
echo -e "${C}Building release binary...${NC}"
cargo install --path "$SRC" --force

# ── Symlink wopr → wopr-2026 for convenience ──
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
ln -sf "$CARGO_BIN/wopr-2026" "$CARGO_BIN/wopr"

# ── Cleanup temp clone ──
[ -n "$CLEANUP" ] && rm -rf "$CLEANUP"

# ── Verify ──
echo ""
if command -v wopr &>/dev/null; then
    echo -e "${G}══════════════════════════════════════════${NC}"
    echo -e "${G}  ✓ Installed!  Run: ${Y}wopr${NC}"
    echo -e "${G}══════════════════════════════════════════${NC}"
else
    echo -e "${Y}Installed to ${CARGO_BIN}/wopr${NC}"
    echo -e "${Y}Add to PATH:  export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
fi
