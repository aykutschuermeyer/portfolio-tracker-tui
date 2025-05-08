use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};
use rust_decimal::Decimal;

use crate::app::portfolio::Portfolio;

pub fn render(frame: &mut Frame, portfolio: &Portfolio, table_state: &mut TableState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let title = Paragraph::new("Portfolio Tracker")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, chunks[0]);

    let holdings = portfolio.holdings();

    if holdings.is_empty() {
        let empty_message = Paragraph::new("No holdings to display. Import transactions first.")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty_message, chunks[1]);
        return;
    }

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

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Positions").borders(Borders::ALL))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, chunks[1], table_state);
}
