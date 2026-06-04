pub mod event;
pub use event::Event;
pub mod key;
pub use key::Key;

use std::{
    sync::mpsc::{Receiver, RecvError, SyncSender, sync_channel},
    time::Duration
};
use anyhow::{Ok, Result};
use crossterm::event::Event as CrosstermEvent;

use crate::{config::Config};

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

/// Unit struct that implements `EventSource`.
/// 
/// Uses `crossterm` types and functions to implement trait functions.
pub struct CrosstermEventSource;
impl EventSource for CrosstermEventSource {
    type Event = CrosstermEvent;

    fn poll(&mut self, timeout: Duration) -> Result<bool> {
        let result = crossterm::event::poll(timeout)?;
        Ok(result)
    }

    fn read(&mut self) -> Result<Self::Event> {
        let event = crossterm::event::read()?;
        Ok(event)
    }
}

/// The project's `Events` structure.
pub struct Events {
    /// The receiving half of mpsc channel.
    rx:  Receiver<Event>,
    /// The sending half of synchronous mpsc channel.
    _tx: SyncSender<Event>
}

impl From<&Config> for Events {
    /// Creates an `Events` struct from the application `Config`.
    /// 
    /// This function extracts event-loop configurations and feeds them directly
    /// into `init()` alongside `CrosstermEventSource`.
    fn from(config: &Config) -> Self {
        let mpsc_channel_capacity = config.event_mpsc_channel_capacity;
        let tick_rate = config.event_tick_rate;
        let exit_key = config.key_config.exit.clone();
        Self::init(mpsc_channel_capacity, tick_rate, exit_key, CrosstermEventSource)
    }
}

impl Events {
    /// This function constructs a synchronous communication channel and moves the generic
    /// `event_source` into a background worker thread where `event_loop()` executes.
    /// 
    /// The background worker thread terminates if call to even_loop() results in an error.
    fn init<E: EventSource + Send + 'static>(
        channel_capacity: usize,
        tick_rate: Duration, exit_key:
        Key, event_source: E
    ) -> Self
    where Event: From<<E as EventSource>::Event>
    {
        let (tx, rx) = sync_channel(channel_capacity);
        let event_tx = tx.clone();

        // Thread terminates when event_loop() returns
        std::thread::spawn(move || -> Result<()> {
            Self::event_loop(event_tx, tick_rate, exit_key, event_source)
        });

        Self {rx, _tx: tx}
    }

    /// The worker thread's event loop.
    /// 
    /// This function blocks on a loop that polls the given `event_source`. If an input 
    /// event occurs within the `tick_rate` duration, it converts and forwards the event. 
    /// Otherwise, it sends a periodic `Event::Tick` to trigger UI updates.
    /// 
    /// # Errors
    /// Returns an error if polling or reading from the `event_source` fails.
    fn event_loop<E: EventSource>(
        event_tx: SyncSender<Event>,
        tick_rate: Duration,
        exit_key: Key,
        mut event_source: E
    ) -> Result<()>
    where Event: From<<E as EventSource>::Event>
    {
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

    /// Blocks the current thread until the next `Event` is available.
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

    impl EventSource for MockEventSource {
        type Event = CrosstermEvent;

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