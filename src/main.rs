use rust_tui_template::app::App;
use rust_tui_template::cli::Args;
use rust_tui_template::config::{Config, setup_config_file};
use rust_tui_template::events::{Event, Events};
use rust_tui_template::terminal::{setup_terminal, restore_terminal};

use ratatui::{backend::Backend, Terminal};
use std::{time::Duration, thread::sleep};
use anyhow::Result;
use log::{info, debug, error};
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();
    info!("Logger initialized");

    // Parse cli args
    let args = Args::parse();

    // Handle debug-only CLI flags
    #[cfg(debug_assertions)]
    handle_build_example_config(&args)?;

    // Setup project config file on system; application can proceed on failure using default config.
    match setup_config_file() {
        Ok(_) => debug!("config file setup successful"),
        Err(e) => error!("config file setup error: {:?}", e)
    }

    // TODO: Leaving off here, document setup_terminal()
    // Setup terminal; application cannot proceed on failure.
    let mut terminal = setup_terminal().inspect_err(|e| error!("terminal setup error: {:?}", e))?;

    // Run app; on both results attempt to restore terminal and exit.
    match run(&mut terminal) {
        Ok(_)  => debug!("run application exited gracefully"),
        Err(e) => error!("run application error: {:?}", e)
    }

    // Restore terminal
    restore_terminal().inspect_err(|e| error!("restore terminal error: {:?}", e))?;

    info!("Exiting program");
    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let config = Config::load().inspect_err(|e| error!("config load error: {:?}", e)).unwrap_or_default();

    let events = Events::from(&config);

    let mut app = App::default();

    loop {
        terminal.draw(|frame| {
            match app.draw(frame) {
                Ok(_state) => {}
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        })?;

        match events.next()? {
            Event::Key(key) => {
                if key == config.key_config.exit {
                    // Give event thread time to exit gracefully
                    sleep(Duration::from_millis(100));
                    break;
                }
            }
            Event::Tick => continue,
            _ => continue
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn handle_build_example_config(args: &Args) -> Result<()> {
    if args.build_example_config {
        Config::default().serialize_config_to_example_and_replace()?;
    }

    Ok(())
}
