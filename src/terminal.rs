use std::io::{self, stdout, Write};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    Basic,
    Colors256,
    TrueColor,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    /// Whether wide CJK chars advance the cursor by exactly 2 cells.
    pub unicode_width_correct: bool,
    // ponytail: TERM_PROGRAM heuristic; actual glyph probe needs a render round-trip
    pub nerd_fonts_likely: bool,
}

pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(out))
}

/// Restore terminal to cooked mode. Best-effort — ignores errors.
pub fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

/// Set a panic hook that restores the terminal before printing panic info.
pub fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        prev(info);
    }));
}

/// Await SIGINT (Ctrl-C) or SIGTERM, restore the terminal, then exit.
///
/// Intended to be spawned with `tokio::spawn(terminal::wait_for_signal())`.
pub async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = sigterm.recv() => {}
            }
        } else {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    restore_terminal();
    std::process::exit(0);
}

/// Probe terminal capabilities and log a one-line summary to stderr.
///
/// Must be called AFTER `init_terminal()` — the cursor position probe needs raw mode.
pub fn detect_capabilities() -> TerminalCapabilities {
    let color_support = probe_color_support();
    let unicode_width_correct = probe_unicode_width();
    let nerd_fonts_likely = probe_nerd_fonts_hint();

    TerminalCapabilities { color_support, unicode_width_correct, nerd_fonts_likely }
}

fn probe_color_support() -> ColorSupport {
    // COLORTERM is authoritative — set by terminals that support 24-bit color
    if let Ok(ct) = std::env::var("COLORTERM") {
        if ct == "truecolor" || ct == "24bit" {
            return ColorSupport::TrueColor;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("truecolor") {
            return ColorSupport::TrueColor;
        }
        if term.contains("256color") {
            return ColorSupport::Colors256;
        }
    }
    // Terminals that reliably ship truecolor support without advertising it via $TERM
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        if matches!(prog.as_str(), "iTerm.app" | "ghostty" | "WezTerm" | "vscode" | "Hyper") {
            return ColorSupport::TrueColor;
        }
    }
    ColorSupport::Basic
}

/// Write a known 2-cell wide CJK char, query cursor before/after, check delta == 2.
fn probe_unicode_width() -> bool {
    let Ok(before) = cursor::position() else {
        return true; // can't probe — assume OK
    };
    let _ = write!(stdout(), "测");
    let _ = stdout().flush();
    let Ok(after) = cursor::position() else {
        return true;
    };
    // Clean up the probe character
    let _ = execute!(
        stdout(),
        cursor::MoveTo(before.0, before.1),
        Clear(ClearType::UntilNewLine),
    );
    after.0.saturating_sub(before.0) == 2
}

fn probe_nerd_fonts_hint() -> bool {
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        return matches!(prog.as_str(), "WezTerm" | "iTerm.app" | "ghostty");
    }
    false
}
