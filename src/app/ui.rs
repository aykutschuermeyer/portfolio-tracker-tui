use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use rust_decimal::Decimal;

use crate::app::portfolio::Portfolio;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render(
    frame: &mut Frame,
    portfolio: &Portfolio,
    table_state: &mut TableState,
    popup_message: &Option<String>,
    error_popup: &Option<String>,
    selection_mode: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Table
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    let title = Paragraph::new("Portfolio Tracker")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, chunks[0]);

    let holdings = portfolio.holdings();

    if holdings.is_empty() {
        let empty_message =
            Paragraph::new("No holdings to display. Press F4 to import transactions.")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty_message, chunks[1]);
    } else {
        let header_cells = [
            "Name",
            // "Symbol",
            "Quantity",
            "Price",
            "Value",
            "Cost",
            "Unr. G/L",
            "Unr. G/L %",
            "Real. G/L",
            "Div.",
            "Total G/L",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
        let header = Row::new(header_cells).style(Style::default()).height(1);

        let rows = holdings.iter().map(|position| {
            let name = position.asset().name();
            // let ticker_symbol = position.asset().tickers()[0].symbol();
            let quantity = format!("{:.2}", position.quantity());
            let price = format!("{:.2}", position.price());
            let market_value = format!("{:.2}", position.market_value());
            let cost_basis = format!("{:.2}", position.total_cost());
            let dividends_collected = format!("{:.2}", position.dividends_collected());

            let unrealized_gain = *position.unrealized_gain();
            let realized_gain = *position.realized_gain();
            let unrealized_gain_percent = *position.unrealized_gain_percent();
            let total_gain = *position.total_gain();

            let color_unrealized_gain = if unrealized_gain >= Decimal::ZERO {
                Color::Green
            } else {
                Color::Red
            };

            let color_realized_gain = if realized_gain >= Decimal::ZERO {
                Color::Green
            } else {
                Color::Red
            };

            let color_unrealized_gain_percent = if unrealized_gain_percent >= Decimal::ZERO {
                Color::Green
            } else {
                Color::Red
            };

            let color_total_gain = if total_gain >= Decimal::ZERO {
                Color::Green
            } else {
                Color::Red
            };

            let unrealized_gain_str = format!("{:.2}", unrealized_gain.abs());
            let urealized_gain_percent_str = format!("{:.2}%", unrealized_gain_percent.abs());
            let realized_gain_str = format!("{:.2}", realized_gain.abs());
            let total_gain_str = format!("{:.2}", total_gain.abs());

            let cells = [
                Cell::from(name.to_string()),
                // Cell::from(ticker_symbol.to_string()),
                Cell::from(quantity),
                Cell::from(price),
                Cell::from(market_value),
                Cell::from(cost_basis),
                Cell::from(unrealized_gain_str).style(Style::default().fg(color_unrealized_gain)),
                Cell::from(urealized_gain_percent_str)
                    .style(Style::default().fg(color_unrealized_gain_percent)),
                Cell::from(realized_gain_str).style(Style::default().fg(color_realized_gain)),
                Cell::from(dividends_collected).style(Style::default().fg(Color::Green)),
                Cell::from(total_gain_str).style(Style::default().fg(color_total_gain)),
            ];

            Row::new(cells).height(1)
        });

        let widths = [
            Constraint::Length(40),
            // Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
        ];

        let mut table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().title("Positions").borders(Borders::ALL));

        if selection_mode {
            table = table.row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        }

        frame.render_stateful_widget(table, chunks[1], table_state);
    }

    let footer = Paragraph::new("F4: Import Transactions | F5: Update Prices | q: Quit")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    // Render loading popup
    if let Some(message) = popup_message {
        let area = centered_rect(50, 20, frame.area());
        let popup = Paragraph::new(message.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("Processing")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow)),
            );
        frame.render_widget(popup, area);
    }

    // Render error popup
    if let Some(error_message) = error_popup {
        let area = centered_rect(60, 25, frame.area());
        let popup = Paragraph::new(format!(
            "{}\n\nPress Enter or Esc to dismiss",
            error_message
        ))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red)),
        );
        frame.render_widget(popup, area);
    }
}
