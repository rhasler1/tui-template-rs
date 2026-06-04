use std::io::{stdout, Stdout};
use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture}
};

/// TODO
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Cleanup on error
    if let Err(e) = execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste) {
        let _ = disable_raw_mode();
        return Err(e.into())
    }

    // Cleanup on error
    if let Err(e) = terminal.clear() {
        // full cleanup
        let _ = restore_terminal();
        return Err(e.into())
    }

    Ok(terminal)
}

/// TODO
pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, DisableBracketedPaste)?;
    
    Ok(())
}