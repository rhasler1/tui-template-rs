use std::sync::mpsc::{Receiver, RecvError, SyncSender, sync_channel};
use std::time::Duration;
use anyhow::{Ok, Result};
use crossterm::event::Event as CrosstermEvent;

use crate::events::key::Key;
use crate::config::Config;

/// TODO
pub trait CrosstermEventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool>;
    fn read(&mut self) -> Result<CrosstermEvent>;
}

pub struct TerminalEventSource;
impl CrosstermEventSource for TerminalEventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool> {
        let result = crossterm::event::poll(timeout)?;
        Ok(result)
    }

    fn read(&mut self) -> Result<CrosstermEvent> {
        let event = crossterm::event::read()?;
        Ok(event)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(Key),
    Paste(String),
    Tick,
    Unknown
}

/*impl Event {
    pub fn is_key(&self, key: Key) -> bool {
        matches!
    }
}*/

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
            _ => Event::Unknown
        }
    }
}

pub struct Events {
    rx:  Receiver<Event>,
    _tx: SyncSender<Event>
}

impl From<&Config> for Events {
    fn from(config: &Config) -> Self {
        let mpsc_channel_capacity = config.event_mpsc_channel_capacity;
        let tick_rate = config.event_tick_rate;
        let exit_key = config.key_config.exit.clone();
        Self::init(mpsc_channel_capacity, tick_rate, exit_key, TerminalEventSource)
    }
}

impl Events {
    fn init<E: CrosstermEventSource + Send + 'static>(channel_capacity: usize, tick_rate: Duration, exit_key: Key, event_source: E) -> Self {
        let (tx, rx) = sync_channel(channel_capacity);
        let event_tx = tx.clone();

        // Thread terminates when event_loop() returns
        std::thread::spawn(move || -> Result<()> {
            Self::event_loop(event_tx, tick_rate, exit_key, event_source)
        });

        Self {rx, _tx: tx}
    }

    fn event_loop<E: CrosstermEventSource>(event_tx: SyncSender<Event>, tick_rate: Duration, exit_key: Key, mut event_source: E) -> Result<()> {
        loop {
            let event = if event_source.poll(tick_rate)? {
                Event::from(event_source.read()?)
            } else {
                Event::Tick
            };

            if event_tx.send(event.clone()).is_err() {
                break;
            }

            let exit_flag = match event {
                Event::Key(inner_key) => inner_key == exit_key,
                _ => false
            };

            if exit_flag {
                break;
            }
        }

        Ok(())
    }

    pub fn next(&self) -> Result<Event, RecvError> {
        self.rx.recv()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    struct MockEventSource {
        events: Vec<CrosstermEvent>
    }

    impl CrosstermEventSource for MockEventSource {
        fn poll(&mut self, _timeout: Duration) -> Result<bool> {
            Ok(!self.events.is_empty())
        }

        fn read(&mut self) -> Result<CrosstermEvent> {
            Ok(self.events.remove(0))
        }
    }

    #[test]
    fn test_event_init() {
        let mock_source = MockEventSource {
            events: vec![CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))]
        };

        let events_struct = Events::init(16, Duration::from_millis(248), Key::Esc, mock_source);


        std::thread::sleep(Duration::from_millis(100));

        let received_event = events_struct.next();
        assert!(received_event.is_ok());
        let received_event = received_event.unwrap();
        let expected_event = Event::Key(Key::Char('a'));

        assert_eq!(received_event, expected_event);

    }
}