use std::{
    io,
    time::{Duration, Instant},
};

use ::tui::{backend::CrosstermBackend, Terminal};
use bot::Bot;
use console::Console;
use crossterm::{
    event::{self, Event},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Result;

mod bot;
mod console;
mod error;
mod input;
mod tui;
//mod save;

const TICK_INTERVAL: Duration = Duration::from_millis(2000);
const RESIZE_BATCH_WAIT_DURATION: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    // =================== LOAD SAVED DATA ===================
    /*let save_data = match save::load_create_save_file() {
        Ok(data) => data,
        Err(err) => panic!("Alertabot Error: {}", err),
    };*/

    // ======================== SETUP ========================
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let mut console = Console::new(terminal)?;
    let mut bot = Bot::new()?;

    // ====================== MAIN LOOP ======================
    let mut last = Instant::now();

    console.render()?;
    loop {
        let elapsed = last.elapsed();
        let timeout = TICK_INTERVAL.checked_sub(elapsed).unwrap_or(Duration::ZERO);

        if event::poll(timeout)? == true {
            match event::read()? {
                event::Event::Key(key) => console.process_input(key),
                event::Event::Resize(..) => {
                    process_resize_batch()?;
                    console.resize()?;
                }
                _ => (),
            }

            if console.should_exit() {
                break;
            }
        }

        // One tick happens every 2 seconds. 1 tick == 2 seconds
        if elapsed >= TICK_INTERVAL {
            last = Instant::now();

            bot.tick();

            console.render()?;
        }
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

/// When the user resizes the terminal, resize events come in batches meaning
/// events returned while resizing the window aren't as important as the last
/// resize event giving us the final terminal dimensions.
fn process_resize_batch() -> Result<()> {
    while let Ok(true) = event::poll(RESIZE_BATCH_WAIT_DURATION) {
        match event::read()? {
            Event::Resize(..) => (),
            _ => break,
        }
    }
    Ok(())
}
