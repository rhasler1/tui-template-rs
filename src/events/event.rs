use crossterm::event::Event as CrosstermEvent;
use crate::events::Key;


#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(Key),
    Paste(String),
    Resize(u16, u16),
    Tick,
    Unknown
}

impl From<CrosstermEvent> for Event {
    fn from(crossterm_event: CrosstermEvent) -> Self {
        match crossterm_event {
            CrosstermEvent::Key(event) => {
                if event.is_press() {
                    Event::Key(Key::from(event))
                } else {
                    Event::Unknown
                }
            },
            CrosstermEvent::Paste(data) => Event::Paste(data),
            CrosstermEvent::Resize(col, row) => Event::Resize(col, row),
            _ => Event::Unknown
        }
    }
}