use crossterm::event::KeyEvent;
use tui::{backend::Backend, Terminal};

use crate::bot::Bot;
use crate::error::Result;
use crate::{
    input::{InputHandler, Interruption},
    tui::TUI,
};

pub struct Console<B: Backend> {
    terminal: Terminal<B>,
    tui: TUI,
    input: InputHandler,
    input_mode: InputMode,

    should_exit: bool,
    //tick: u16,
}

impl<B: Backend> Console<B> {
    //const TICKS_PER_UPDATE: u16 = 1;

    pub fn new(terminal: Terminal<B>) -> Result<Self> {
        let mut tui = TUI::new();
        tui.resize(terminal.size()?);
        Ok(Self {
            terminal,
            tui,
            input: InputHandler::new(),
            input_mode: InputMode::Editing,

            should_exit: false,
            //tick: 0,
        })
    }

    /// Increments the inner ticker and schedules TUI updates per `TICKS_PER_UPDATE`.
    pub fn update(&mut self, bot: &Bot) {
        //self.tick += 1;
        //
        //if self.tick >= Self::TICKS_PER_UPDATE {
        //    self.update_tui(bot)
        //    self.tick = 0;
        //};
        self.update_tui(bot);
    }

    fn update_tui(&mut self, bot: &Bot) {
        self.tui.update(bot)
    }

    pub fn process_input(&mut self, event: KeyEvent) {
        match self.input_mode {
            InputMode::Editing => self.process_editing(event),
            InputMode::Control => self.process_controls(event),
        }
    }

    fn process_controls(&mut self, event: KeyEvent) {}

    fn process_editing(&mut self, event: KeyEvent) {
        if let Some(interruption) = self.input.process_input(event) {
            match interruption {
                Interruption::Enter(buf) => println!("You entered: {buf}"),
                Interruption::Esc => self.should_exit = true,
            }
        }
    }

    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|frame| self.tui.render(frame))?;
        Ok(())
    }

    pub fn resize(&mut self) -> Result<()> {
        self.tui.resize(self.terminal.size()?);
        Ok(())
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Editing,
    Control,
}
