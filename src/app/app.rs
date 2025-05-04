use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::TableState,
    Terminal,
};

use crate::app::{ui, Portfolio};

pub struct App {
    portfolio: Portfolio,
    table_state: TableState,
}

impl App {
    pub fn new(portfolio: Portfolio) -> Self {
        let mut table_state = TableState::default();
        if !portfolio.holdings().is_empty() {
            table_state.select(Some(0));
        }

        Self {
            portfolio,
            table_state,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, &self.portfolio, &mut self.table_state))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        let holdings = self.portfolio.holdings();
                        if !holdings.is_empty() {
                            let i = match self.table_state.selected() {
                                Some(i) => {
                                    if i >= holdings.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            self.table_state.select(Some(i));
                        }
                    }
                    KeyCode::Up => {
                        let holdings = self.portfolio.holdings();
                        if !holdings.is_empty() {
                            let i = match self.table_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        holdings.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            self.table_state.select(Some(i));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
