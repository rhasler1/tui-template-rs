# A starting point for tui applications in Rust
This project depends on `ratatui` for rendering and `crossterm` for terminal manipulation.
Most of the underlying behaviors of `crossterm` and `ratatui` are left unabstracted.
For example, in `terminal.rs`, terminal initialization--such as entering raw mode and entering an alternate screen--uses concrete types directly:

```rust
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
```


In `events.rs`, the functions `poll()` and `read()` are abstracted behind the `EventSource` trait. This abstraction is introduced solely to enable mock testing of the `Config::init()` constructor and `Config::event_loop()` associated function without requiring a live terminal session.

```rust
/// Trait used to mock polling and reading trait type: `EventSource::Event`.
/// 
/// This trait abstracts the standard blocking I/O functions found in `crossterm::event`
/// to allow for deterministic unit testing and mocking of terminal input.
pub trait EventSource {
    /// The specific type of event emitted by this source.
    type Event;
    /// Checks if a terminal event is available within a given timeout duration.
    fn poll(&mut self, timeout: Duration) -> Result<bool>;
    /// Reads a single event from the input buffer.
    fn read(&mut self) -> Result<Self::Event>;
}
```
