use std::io::Stdout;
use std::io::stdout;
use std::panic;

use ratatui::Terminal as RatatuiTerminal;
use ratatui::prelude::CrosstermBackend;

use crossterm::ExecutableCommand;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;

use color_eyre::eyre::WrapErr;

pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

pub fn init_terminal() -> color_eyre::Result<Terminal> {
    enable_raw_mode().wrap_err("Failed to enable raw mode")?;

    stdout()
        .execute(EnterAlternateScreen)
        .wrap_err("Failed to enter alternate screen")?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .wrap_err("Failed to start new terminal")?;

    Ok(terminal)
}

pub fn restore() -> color_eyre::Result<()> {
    if let Err(e) = stdout().execute(LeaveAlternateScreen) {
        eprintln!("Cleanup failed leaving alternate screen: {e}");
    }

    if let Err(e) = disable_raw_mode() {
        eprintln!("Cleanup failed to disable raw mode: {e}");
    }

    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if let Err(e) = restore() {
            eprintln!("Failed to restore terminal. {}", e);
        }
        original_hook(panic_info);
    }));
}
