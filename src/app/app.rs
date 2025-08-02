use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    widgets::TableState,
};

use crate::app::{Portfolio, ui};

pub struct App {
    portfolio: Portfolio,
    table_state: TableState,
    popup_message: Option<String>,
    error_popup: Option<String>,
    selection_mode: bool,
}

impl App {
    pub fn new(portfolio: Portfolio) -> Self {
        Self {
            portfolio,
            table_state: TableState::default(),
            popup_message: None,
            error_popup: None,
            selection_mode: false,
        }
    }

    fn show_popup(&mut self, message: &str) {
        self.popup_message = Some(message.to_string());
    }

    fn clear_popup(&mut self) {
        self.popup_message = None;
    }

    fn show_error_popup(&mut self, message: &str) {
        self.error_popup = Some(message.to_string());
    }

    fn clear_error_popup(&mut self) {
        self.error_popup = None;
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|frame| {
                ui::render(
                    frame,
                    &self.portfolio,
                    &mut self.table_state,
                    &self.popup_message,
                    &self.error_popup,
                    self.selection_mode,
                )
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Enter | KeyCode::Esc => {
                        if self.error_popup.is_some() {
                            self.clear_error_popup();
                            continue;
                        }
                        if key.code == KeyCode::Esc {
                            self.selection_mode = false;
                            self.table_state.select(None);
                        }
                    }
                    KeyCode::F(4) => {
                        self.selection_mode = false;
                        self.table_state.select(None);
                        self.show_popup("Importing transactions...");
                        terminal.draw(|frame| {
                            ui::render(
                                frame,
                                &self.portfolio,
                                &mut self.table_state,
                                &self.popup_message,
                                &self.error_popup,
                                self.selection_mode,
                            )
                        })?;

                        let csv_path =
                            shellexpand::tilde("~/.config/portfolio-tracker-tui/transactions.csv");
                        let import_result = self.portfolio.import_transactions(&csv_path).await;
                        let update_result = self.portfolio.update_prices().await;
                        let holdings_result = self.portfolio.set_holdings().await;

                        self.clear_popup();
                        terminal.draw(|frame| {
                            ui::render(
                                frame,
                                &self.portfolio,
                                &mut self.table_state,
                                &self.popup_message,
                                &self.error_popup,
                                self.selection_mode,
                            )
                        })?;

                        if let Err(e) = import_result {
                            self.show_error_popup(&format!("Error importing transactions: {}", e));
                        } else if let Err(e) = update_result {
                            self.show_error_popup(&format!("Error updating prices: {}", e));
                        } else if let Err(e) = holdings_result {
                            self.show_error_popup(&format!("Error updating holdings: {}", e));
                        }
                    }
                    KeyCode::F(5) => {
                        self.selection_mode = false;
                        self.table_state.select(None);
                        self.show_popup("Updating prices...");
                        terminal.draw(|frame| {
                            ui::render(
                                frame,
                                &self.portfolio,
                                &mut self.table_state,
                                &self.popup_message,
                                &self.error_popup,
                                self.selection_mode,
                            )
                        })?;

                        let update_result = self.portfolio.update_prices().await;
                        let holdings_result = self.portfolio.set_holdings().await;

                        self.clear_popup();
                        terminal.draw(|frame| {
                            ui::render(
                                frame,
                                &self.portfolio,
                                &mut self.table_state,
                                &self.popup_message,
                                &self.error_popup,
                                self.selection_mode,
                            )
                        })?;

                        if let Err(e) = update_result {
                            self.show_error_popup(&format!("Error updating prices: {}", e));
                        } else if let Err(e) = holdings_result {
                            self.show_error_popup(&format!("Error updating holdings: {}", e));
                        }
                    }
                    KeyCode::Down => {
                        self.selection_mode = true;
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
                        self.selection_mode = true;
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
