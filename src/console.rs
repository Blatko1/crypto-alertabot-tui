use crossterm::event::KeyEvent;
use tui::{backend::Backend, Terminal};

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
}

impl<B: Backend> Console<B> {
    pub fn new(terminal: Terminal<B>) -> Result<Self> {
        let mut tui = TUI::new();
        tui.resize(terminal.size()?);
        Ok(Self {
            terminal,
            tui,
            input: InputHandler::new(),
            input_mode: InputMode::Editing,

            should_exit: false,
        })
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
