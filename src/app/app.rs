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

trait SelectableState {
    fn selected(&self) -> Option<usize>;
    fn select(&mut self, index: Option<usize>);
}

impl SelectableState for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn select(&mut self, index: Option<usize>) {
        self.select(index);
    }
}

impl SelectableState for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn select(&mut self, index: Option<usize>) {
        self.select(index);
    }
}

struct PopupManager {
    message: Option<String>,
    error: Option<String>,
    show_api_selector: bool,
    show_database_reset: bool,
}

impl PopupManager {
    fn new() -> Self {
        Self {
            message: None,
            error: None,
            show_api_selector: false,
            show_database_reset: false,
        }
    }

    fn show_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    fn clear_message(&mut self) {
        self.message = None;
    }

    fn show_error(&mut self, message: &str) {
        self.error = Some(message.to_string());
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    fn has_error(&self) -> bool {
        self.error.is_some()
    }

    fn has_any_popup(&self) -> bool {
        self.show_api_selector || self.show_database_reset
    }
}

pub struct App {
    portfolio: Portfolio,
    table_state: TableState,
    popup_manager: PopupManager,
    default_api_state: ListState,
    selection_mode: bool,
    default_reset_state: ListState,
}

impl App {
    pub fn new(portfolio: Portfolio) -> Self {
        let mut default_api_list_state = ListState::default();
        default_api_list_state.select(Some(0));
        let mut default_reset_list_state = ListState::default();
        default_reset_list_state.select(Some(0));
        Self {
            portfolio,
            table_state: TableState::default(),
            popup_manager: PopupManager::new(),
            default_api_state: default_api_list_state,
            selection_mode: false,
            default_reset_state: default_reset_list_state,
        }
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

    fn render_ui<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.draw(|frame| {
            ui::render(
                frame,
                &self.portfolio,
                &mut self.table_state,
                &self.popup_manager.message,
                &self.popup_manager.error,
                self.popup_manager.show_api_selector,
                &mut self.default_api_state,
                self.selection_mode,
                self.popup_manager.show_database_reset,
                &mut self.default_reset_state,
            )
        })?;
        Ok(())
    }

    fn deselect_table(&mut self) {
        self.selection_mode = false;
        self.table_state.select(None);
    }

    fn navigate_up<T: SelectableState>(state: &mut T, list_len: usize) {
        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    list_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    fn navigate_down<T: SelectableState>(state: &mut T, list_len: usize) {
        let i = match state.selected() {
            Some(i) => {
                if i >= list_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    fn handle_api_popup_keys(&mut self, key_code: KeyCode) -> Result<()> {
        self.deselect_table();
        match key_code {
            KeyCode::Esc => {
                self.popup_manager.show_api_selector = false;
            }
            KeyCode::Down => {
                Self::navigate_down(&mut self.default_api_state, ApiProvider::iter().len());
            }
            KeyCode::Up => {
                Self::navigate_up(&mut self.default_api_state, ApiProvider::iter().len());
            }
            KeyCode::Enter => {
                if let Some(i) = self.default_api_state.selected() {
                    self.portfolio.set_default_api(
                        ApiProvider::iter()
                            .nth(i)
                            .with_context(|| "Cannot select API provider")?,
                    );
                    self.popup_manager.show_api_selector = false;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_reset_popup_keys<B: Backend>(
        &mut self,
        key_code: KeyCode,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        self.deselect_table();
        match key_code {
            KeyCode::Esc => {
                self.popup_manager.show_database_reset = false;
            }
            KeyCode::Down => {
                Self::navigate_down(&mut self.default_reset_state, 3);
            }
            KeyCode::Up => {
                Self::navigate_up(&mut self.default_reset_state, 3);
            }
            KeyCode::Enter => {
                match self.default_reset_state.selected() {
                    Some(0) => {
                        // Cancel
                        self.popup_manager.show_database_reset = false;
                        self.selection_mode = true;
                    }
                    Some(1) => {
                        // Clear transactions and holdings
                        self.portfolio.reset(false).await?;
                        self.portfolio.set_holdings().await?;
                        self.popup_manager.show_database_reset = false;
                        self.selection_mode = true;
                        self.render_ui(terminal)?;
                    }
                    Some(2) => {
                        // Clear everything including tickers
                        self.portfolio.reset(true).await?;
                        self.portfolio.set_holdings().await?;
                        self.popup_manager.show_database_reset = false;
                        self.selection_mode = true;
                        self.render_ui(terminal)?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_table_navigation(&mut self, key_code: KeyCode) {
        if !self.popup_manager.has_any_popup() {
            self.selection_mode = true;
        }
        let holdings = self.portfolio.holdings();
        if holdings.is_empty() {
            return;
        }

        match key_code {
            KeyCode::Down => {
                Self::navigate_down(&mut self.table_state, holdings.len());
            }
            KeyCode::Up => {
                Self::navigate_up(&mut self.table_state, holdings.len());
            }
            _ => {}
        }
    }

    async fn import_transactions<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        csv_path: &str,
    ) -> Result<()> {
        self.deselect_table();
        self.popup_manager.show_message("Importing transactions...");
        self.render_ui(terminal)?;

        let csv_path_expanded = shellexpand::tilde(csv_path);
        let default_api = self.portfolio.default_api().clone();

        let import_result = self
            .portfolio
            .import_transactions(&csv_path_expanded, &default_api)
            .await;
        let update_result = self.portfolio.update_prices().await;
        let holdings_result = self.portfolio.set_holdings().await;

        self.popup_manager.clear_message();
        self.render_ui(terminal)?;

        if let Err(e) = import_result {
            self.popup_manager
                .show_error(&format!("Error importing transactions: {:?}", e));
        } else if let Err(e) = update_result {
            self.popup_manager
                .show_error(&format!("Error updating prices: {:?}", e));
        } else if let Err(e) = holdings_result {
            self.popup_manager
                .show_error(&format!("Error updating holdings: {:?}", e));
        }

        Ok(())
    }

    async fn update_prices<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.deselect_table();
        self.popup_manager.show_message("Updating prices...");
        self.render_ui(terminal)?;

        let update_result = self.portfolio.update_prices().await;
        let holdings_result = self.portfolio.set_holdings().await;

        self.popup_manager.clear_message();
        self.render_ui(terminal)?;

        if let Err(e) = update_result {
            self.popup_manager
                .show_error(&format!("Error updating prices: {:?}", e));
        } else if let Err(e) = holdings_result {
            self.popup_manager
                .show_error(&format!("Error updating holdings: {:?}", e));
        }

        Ok(())
    }

    async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        csv_path: &str,
    ) -> Result<()> {
        loop {
            self.render_ui(terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.popup_manager.show_api_selector {
                    self.handle_api_popup_keys(key.code)?;
                    continue;
                }

                if self.popup_manager.show_database_reset {
                    self.handle_reset_popup_keys(key.code, terminal).await?;
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Enter | KeyCode::Esc => {
                        if self.popup_manager.has_error() {
                            self.popup_manager.clear_error();
                            continue;
                        }
                        if key.code == KeyCode::Esc {
                            self.deselect_table();
                        }
                    }
                    KeyCode::F(4) => {
                        self.import_transactions(terminal, csv_path).await?;
                    }
                    KeyCode::F(5) => {
                        self.update_prices(terminal).await?;
                    }
                    KeyCode::F(8) => {
                        self.deselect_table();
                        self.popup_manager.show_api_selector = true;
                    }
                    KeyCode::F(12) => {
                        self.deselect_table();
                        self.popup_manager.show_database_reset = true;
                    }
                    KeyCode::Down | KeyCode::Up => {
                        self.handle_table_navigation(key.code);
                    }
                    _ => {}
                }
            }
        }
    }
}
