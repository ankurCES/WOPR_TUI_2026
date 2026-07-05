#!/usr/bin/env bash
set -euo pipefail

# ── Colors ──
G='\033[0;32m'; Y='\033[1;33m'; C='\033[0;36m'; R='\033[0;31m'
DIM='\033[2m'; BOLD='\033[1m'; NC='\033[0m'
BG='\033[0;32m' # green-on-black CRT vibe

# ── Typewriter effect ──
typewrite() {
    local text="$1" delay="${2:-0.03}"
    for (( i=0; i<${#text}; i++ )); do
        printf '%s' "${text:$i:1}"
        sleep "$delay"
    done
    echo ""
}

typewrite_color() {
    local color="$1" text="$2" delay="${3:-0.03}"
    printf '%b' "$color"
    for (( i=0; i<${#text}; i++ )); do
        printf '%s' "${text:$i:1}"
        sleep "$delay"
    done
    printf '%b\n' "$NC"
}

# ── Spinner ──
spinner() {
    local pid=$1 msg="$2"
    local frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r  %b%s%b %s" "$C" "${frames[$i]}" "$NC" "$msg"
        i=$(( (i + 1) % ${#frames[@]} ))
        sleep 0.1
    done
    wait "$pid" 2>/dev/null
    local rc=$?
    if [ $rc -eq 0 ]; then
        printf "\r  %b✓%b %s\n" "$G" "$NC" "$msg"
    else
        printf "\r  %b✗%b %s\n" "$R" "$NC" "$msg"
    fi
    return $rc
}

# ── Clear screen, CRT boot ──
clear
sleep 0.3

echo ""
typewrite_color "$G" "    ╔══════════════════════════════════════════════════════════╗" 0.008
typewrite_color "$G" "    ║                                                          ║" 0.008
typewrite_color "$G" "    ║   ██╗    ██╗ ██████╗ ██████╗ ██████╗                     ║" 0.008
typewrite_color "$G" "    ║   ██║    ██║██╔═══██╗██╔══██╗██╔══██╗                    ║" 0.008
typewrite_color "$G" "    ║   ██║ █╗ ██║██║   ██║██████╔╝██████╔╝                    ║" 0.008
typewrite_color "$G" "    ║   ██║███╗██║██║   ██║██╔═══╝ ██╔══██╗                    ║" 0.008
typewrite_color "$G" "    ║   ╚███╔███╔╝╚██████╔╝██║     ██║  ██║                    ║" 0.008
typewrite_color "$G" "    ║    ╚══╝╚══╝  ╚═════╝ ╚═╝     ╚═╝  ╚═╝                    ║" 0.008
typewrite_color "$G" "    ║                                                          ║" 0.008
typewrite_color "$G" "    ║       War Operation Plan Response — NORAD, Cheyenne Mtn  ║" 0.008
typewrite_color "$G" "    ║                                                          ║" 0.008
typewrite_color "$G" "    ╚══════════════════════════════════════════════════════════╝" 0.008
echo ""
sleep 0.5

typewrite_color "$DIM" "    WOPR MAINFRAME — SYSCONN 4.0.19  [PRIMARY SURVEILLANCE]" 0.02
typewrite_color "$DIM" "    AUTHORIZED ACCESS ONLY — 18 U.S.C. § 1030" 0.02
echo ""
sleep 0.4

typewrite_color "$G" "    GREETINGS." 0.06
echo ""
sleep 0.5

# ── Login loop ──
MAX_ATTEMPTS=3
ATTEMPT=0
LOGGED_IN=false

while [ "$ATTEMPT" -lt "$MAX_ATTEMPTS" ]; do
    printf "    %bLOGON:%b  " "$Y" "$NC"
    read -r LOGIN </dev/tty

    if [ -z "$LOGIN" ]; then
        continue
    fi

    ATTEMPT=$((ATTEMPT + 1))

    # Accept Joshua (case-insensitive)
    if echo "$LOGIN" | grep -iq "^joshua$"; then
        LOGGED_IN=true
        break
    fi

    echo ""
    sleep 0.3
    typewrite_color "$R" "    IDENTIFICATION NOT RECOGNIZED BY SYSTEM" 0.02
    typewrite_color "$R" "    --ERROR--  INVALID LOGON" 0.02
    echo ""
    sleep 0.5

    if [ "$ATTEMPT" -ge "$MAX_ATTEMPTS" ]; then
        typewrite_color "$R" "    *** TERMINAL LOCKED ***" 0.04
        typewrite_color "$R" "    NORAD SECURITY NOTIFIED — DISCONNECTING" 0.03
        echo ""
        sleep 1
        exit 1
    fi
done

if [ "$LOGGED_IN" != "true" ]; then
    exit 1
fi

# ── Welcome sequence ──
echo ""
sleep 0.6

echo ""
typewrite_color "$G" "    HELLO, PROFESSOR FALKEN." 0.06
sleep 0.8
echo ""
typewrite_color "$C" "    A STRANGE GAME." 0.06
sleep 0.5
typewrite_color "$C" "    THE ONLY WINNING MOVE IS NOT TO PLAY." 0.05
sleep 0.8
echo ""
typewrite_color "$Y" "    ...HOW ABOUT A NICE GAME OF GLOBAL THERMONUCLEAR WAR?" 0.04
sleep 1.2
echo ""

typewrite_color "$G" "    INITIATING WOPR TUI 2026 INSTALLATION SEQUENCE..." 0.03
echo ""
sleep 0.5

# ── Fake missile-launch-style countdown ──
STEPS=(
    "ACCESSING NORAD MAINFRAME.............."
    "DECRYPTING LAUNCH CODES................"
    "BYPASSING SECURITY PROTOCOLS..........."
    "ESTABLISHING SATELLITE UPLINK.........."
    "LOADING DEFCON STATUS MATRIX..........."
    "INITIALIZING THEATER DISPLAY..........."
)

for step in "${STEPS[@]}"; do
    printf "    %b>%b " "$G" "$NC"
    for (( i=0; i<${#step}; i++ )); do
        printf '%s' "${step:$i:1}"
        sleep 0.012
    done
    sleep 0.2
    printf " %b[OK]%b\n" "$G" "$NC"
    sleep 0.15
done

echo ""
typewrite_color "$G" "    ═══════════════════════════════════════════" 0.01
typewrite_color "$G" "     WOPR SYSTEM ONLINE — BEGINNING DEPLOYMENT" 0.02
typewrite_color "$G" "    ═══════════════════════════════════════════" 0.01
echo ""
sleep 0.5

# ════════════════════════════════════════════════════════════
#  REAL INSTALLER BEGINS
# ════════════════════════════════════════════════════════════

OS="$(uname -s)"

# ── Rust ──
if ! command -v cargo &>/dev/null; then
    echo ""
    typewrite_color "$Y" "    RUST TOOLCHAIN NOT DETECTED — INSTALLING VIA RUSTUP..." 0.02
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>/dev/null &
    spinner $! "Installing Rust toolchain"
    source "$HOME/.cargo/env"
fi
printf "    %b✓%b Rust %s\n" "$G" "$NC" "$(rustc --version | cut -d' ' -f2)"

# ── macOS: Xcode CLI tools ──
if [ "$OS" = "Darwin" ] && ! xcode-select -p &>/dev/null; then
    typewrite_color "$C" "    INSTALLING XCODE COMMAND LINE TOOLS..." 0.02
    xcode-select --install
    until xcode-select -p &>/dev/null; do sleep 5; done
fi

# ── Linux: cc + pkg-config + OpenSSL headers ──
if [ "$OS" = "Linux" ]; then
    MISSING=""
    command -v cc         &>/dev/null || MISSING="$MISSING build-essential"
    command -v pkg-config &>/dev/null || MISSING="$MISSING pkg-config"
    [ -f /usr/include/openssl/ssl.h ] || [ -f /usr/include/x86_64-linux-gnu/openssl/ssl.h ] || MISSING="$MISSING libssl-dev"
    if [ -n "$MISSING" ]; then
        typewrite_color "$C" "    INSTALLING SYSTEM DEPENDENCIES:${MISSING}" 0.02
        if   command -v apt-get &>/dev/null; then sudo apt-get update -qq && sudo apt-get install -y $MISSING
        elif command -v dnf     &>/dev/null; then sudo dnf install -y ${MISSING//build-essential/gcc} ${MISSING//libssl-dev/openssl-devel}
        elif command -v pacman  &>/dev/null; then sudo pacman -Sy --noconfirm ${MISSING//build-essential/base-devel} ${MISSING//libssl-dev/openssl}
        else echo "    ERROR: Install manually:$MISSING"; exit 1; fi
    fi
fi

# ── Source: local repo or clone ──
CLEANUP=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}" 2>/dev/null)" && pwd 2>/dev/null)" || SCRIPT_DIR=""
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    SRC="$SCRIPT_DIR"
    printf "    %b✓%b Source: local repo\n" "$G" "$NC"
else
    SRC="$(mktemp -d)"
    CLEANUP="$SRC"
    git clone --depth 1 https://github.com/ankurCES/WOPR_TUI_2026.git "$SRC" 2>/dev/null &
    spinner $! "Cloning WOPR TUI 2026 from NORAD archives"
    SRC="$SRC/WOPR_TUI_2026"
fi

# ── Build ──
echo ""
typewrite_color "$Y" "    COMPILING WOPR THEATER ENGINE..." 0.025
echo ""
BUILD_LOG=$(mktemp)
cargo install --path "$SRC" --force >"$BUILD_LOG" 2>&1 &
if ! spinner $! "Building release binary (this takes 1-2 minutes)"; then
    echo ""
    printf "  %b✗ BUILD FAILED. Last 50 lines:%b\n" "$R" "$NC"
    tail -50 "$BUILD_LOG"
    rm -f "$BUILD_LOG"
    exit 1
fi
rm -f "$BUILD_LOG"

# ── Symlink wopr → wopr-2026 ──
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
ln -sf "$CARGO_BIN/wopr-2026" "$CARGO_BIN/wopr"

# ── Cleanup ──
[ -n "$CLEANUP" ] && rm -rf "$CLEANUP"

# ── Victory screen ──
echo ""
sleep 0.3
echo ""
typewrite_color "$G" "    ╔══════════════════════════════════════════════════════════╗" 0.006
typewrite_color "$G" "    ║                                                          ║" 0.006
typewrite_color "$G" "    ║              WOPR TUI 2026 — INSTALLED                   ║" 0.006
typewrite_color "$G" "    ║                                                          ║" 0.006
typewrite_color "$G" "    ║   DEFCON STATUS .............. 5 (PEACETIME)              ║" 0.006
typewrite_color "$G" "    ║   THEATER DISPLAY ............ ONLINE                    ║" 0.006
typewrite_color "$G" "    ║   AI STRATEGY ENGINE ......... ACTIVE                    ║" 0.006
typewrite_color "$G" "    ║   LAUNCH CAPABILITY .......... ARMED                     ║" 0.006
typewrite_color "$G" "    ║                                                          ║" 0.006
sleep 0.3
if command -v wopr &>/dev/null; then
    typewrite_color "$Y" "    ║   >>> TO BEGIN: type 'wopr' and press ENTER <<<          ║" 0.006
else
    typewrite_color "$Y" "    ║   >>> Add ~/.cargo/bin to PATH, then run: wopr <<<       ║" 0.006
fi
typewrite_color "$G" "    ║                                                          ║" 0.006
typewrite_color "$G" "    ╚══════════════════════════════════════════════════════════╝" 0.006
echo ""
typewrite_color "$DIM" "    SHALL WE PLAY A GAME?" 0.06
echo ""
