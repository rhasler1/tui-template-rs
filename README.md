# A starting point for tui applications in Rust
This project depends on `ratatui` for rendering and `crossterm` for terminal manipulation.
Most of the underlying behaviors of `crossterm` and `ratatui` are left unabstracted.
For example, in `terminal.rs`, terminal initialization--such as entering raw mode and entering an alternate screen--uses concrete types directly:

```rust
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
    terminal.clear()?;

    Ok(terminal)
}
```


In `events.rs`, the crossterm functions `poll()` and `read()` are abstracted behind the `CrosstermEventSource` trait. This abstraction is introduced solely to enable mock testing of the `Config::init()` constructor and `Config::event_loop()` associated function without requiring a live terminal session.

```rust
pub trait CrosstermEventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool>;
    fn read(&mut self) -> Result<CrosstermEvent>;
}
```
