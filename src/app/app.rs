use std::io;
use strum::IntoEnumIterator;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    widgets::{ListState, TableState},
};

use crate::{
    app::{Portfolio, ui},
    models::ticker::ApiProvider,
};

pub struct App {
    portfolio: Portfolio,
    table_state: TableState,
    popup_message: Option<String>,
    error_popup: Option<String>,
    show_api_popup: bool,
    default_api_state: ListState,
    selection_mode: bool,
}

impl App {
    pub fn new(portfolio: Portfolio) -> Self {
        let mut default_api_list_state = ListState::default();
        default_api_list_state.select(Some(0));
        Self {
            portfolio,
            table_state: TableState::default(),
            popup_message: None,
            error_popup: None,
            show_api_popup: false,
            default_api_state: default_api_list_state,
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

    pub async fn run(&mut self, csv_path: &str) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal, csv_path).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        csv_path: &str,
    ) -> Result<()> {
        loop {
            terminal.draw(|frame| {
                ui::render(
                    frame,
                    &self.portfolio,
                    &mut self.table_state,
                    &self.popup_message,
                    &self.error_popup,
                    self.show_api_popup,
                    &mut self.default_api_state,
                    self.selection_mode,
                )
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.show_api_popup {
                    self.selection_mode = false;
                    self.table_state.select(None);
                    match key.code {
                        KeyCode::Esc => self.show_api_popup = false,
                        KeyCode::Down => {
                            let i = match self.default_api_state.selected() {
                                Some(i) => {
                                    if i >= ApiProvider::iter().len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            self.default_api_state.select(Some(i));
                        }
                        KeyCode::Up => {
                            let i = match self.default_api_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        ApiProvider::iter().len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            self.default_api_state.select(Some(i));
                        }
                        KeyCode::Enter => {
                            if let Some(i) = self.default_api_state.selected() {
                                self.default_api_state.select(Some(i));
                                self.portfolio.set_default_api(
                                    ApiProvider::iter()
                                        .nth(i)
                                        .with_context(|| "Cannot select")?,
                                );

                                self.show_api_popup = false;
                            }
                        }
                        _ => {}
                    }
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
                                self.show_api_popup,
                                &mut self.default_api_state,
                                self.selection_mode,
                            )
                        })?;

                        let csv_path = shellexpand::tilde(csv_path);

                        let import_result = self
                            .portfolio
                            .import_transactions(&csv_path, &self.portfolio.default_api().clone())
                            .await;
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
                                self.show_api_popup,
                                &mut self.default_api_state,
                                self.selection_mode,
                            )
                        })?;

                        if let Err(e) = import_result {
                            self.show_error_popup(&format!(
                                "Error importing transactions: {:?}",
                                e
                            ));
                        } else if let Err(e) = update_result {
                            self.show_error_popup(&format!("Error updating prices: {:?}", e));
                        } else if let Err(e) = holdings_result {
                            self.show_error_popup(&format!("Error updating holdings: {:?}", e));
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
                                self.show_api_popup,
                                &mut self.default_api_state,
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
                                self.show_api_popup,
                                &mut self.default_api_state,
                                self.selection_mode,
                            )
                        })?;

                        if let Err(e) = update_result {
                            self.show_error_popup(&format!("Error updating prices: {:?}", e));
                        } else if let Err(e) = holdings_result {
                            self.show_error_popup(&format!("Error updating holdings: {:?}", e));
                        }
                    }
                    KeyCode::F(8) => {
                        self.selection_mode = false;
                        self.show_api_popup = true;
                    }
                    KeyCode::Down => {
                        if !self.show_api_popup {
                            self.selection_mode = true;
                        }
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
                        if !self.show_api_popup {
                            self.selection_mode = true;
                        }
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
