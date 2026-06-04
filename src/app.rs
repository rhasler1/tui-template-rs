use ratatui::prelude::{Frame,Layout,Constraint,Span};

use crate::config::Config;
use crate::events::Key;

pub struct App {
    message: String,
    exit_key: Key
}

impl App {
    pub fn with_config(config: &Config) -> Self {   
        Self {
            message: String::from("Hello"),
            exit_key: config.key_config.exit.clone()
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let height = frame.size().height as usize;
        let constraints: Vec<Constraint> = vec![Constraint::Length(1); height];
        let row_chunks = Layout::vertical(constraints).split(frame.size());
        for (i, &chunk) in row_chunks.iter().enumerate() {
            let text = format!("This is horizontal row #{}", i);
            let span = Span::raw(text);
            frame.render_widget(span, chunk);
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        let message: String = String::from("Hello, press 'q' or left click to exit.");
        Self {
            message,
            exit_key: Key::Esc
        }
    }
}
