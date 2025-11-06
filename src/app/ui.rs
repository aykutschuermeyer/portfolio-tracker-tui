use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};
use rust_decimal::Decimal;
use strum::IntoEnumIterator;

use crate::{app::portfolio::Portfolio, models::ticker::ApiProvider};

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

fn gain_color(value: Decimal) -> Color {
    if value >= Decimal::ZERO {
        Color::Green
    } else {
        Color::Red
    }
}

fn format_colored_gain(value: Decimal) -> (String, Color) {
    (format!("{:.2}", value.abs()), gain_color(value))
}

fn format_colored_percentage(value: Decimal) -> (String, Color) {
    (format!("{:.2}%", value.abs()), gain_color(value))
}

fn render_title(frame: &mut Frame, portfolio: &Portfolio, area: Rect) {
    let title = Paragraph::new(format!(
        "Portfolio Tracker (default API: {})",
        portfolio.default_api().to_str()
    ))
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(concat!(
        "F4: Import Transactions | ",
        "F5: Update Prices | ",
        "F8: Change default API | ",
        "F12: Reset | ",
        "Q: Quit",
    ))
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn render_holdings_table(
    frame: &mut Frame,
    portfolio: &Portfolio,
    table_state: &mut TableState,
    selection_mode: bool,
    area: Rect,
) {
    let holdings = portfolio.holdings();

    if holdings.is_empty() {
        let empty_message =
            Paragraph::new("No holdings to display. Press F4 to import transactions.")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty_message, area);
        return;
    }

    let header_cells = [
        "Name",
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
        let (unrealized_gain_str, color_unrealized) =
            format_colored_gain(*position.unrealized_gain());
        let (unrealized_percent_str, color_unrealized_percent) =
            format_colored_percentage(*position.unrealized_gain_percent());
        let (realized_gain_str, color_realized) = format_colored_gain(*position.realized_gain());
        let (total_gain_str, color_total) = format_colored_gain(*position.total_gain());

        let cells = [
            Cell::from(position.asset().name().to_string()),
            Cell::from(format!("{:.2}", position.quantity())),
            Cell::from(format!("{:.2}", position.price())),
            Cell::from(format!("{:.2}", position.market_value())),
            Cell::from(format!("{:.2}", position.total_cost())),
            Cell::from(unrealized_gain_str).style(Style::default().fg(color_unrealized)),
            Cell::from(unrealized_percent_str).style(Style::default().fg(color_unrealized_percent)),
            Cell::from(realized_gain_str).style(Style::default().fg(color_realized)),
            Cell::from(format!("{:.2}", position.dividends_collected()))
                .style(Style::default().fg(Color::Green)),
            Cell::from(total_gain_str).style(Style::default().fg(color_total)),
        ];

        Row::new(cells).height(1)
    });

    let widths = [
        Constraint::Length(50),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(11),
    ];

    let mut table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Positions").borders(Borders::ALL));

    if selection_mode {
        table = table.row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    frame.render_stateful_widget(table, area, table_state);
}

fn render_message_popup(frame: &mut Frame, message: &str) {
    let area = centered_rect(50, 20, frame.area());
    let popup = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("Processing")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        );
    frame.render_widget(popup, area);
}

fn render_error_popup(frame: &mut Frame, error_message: &str) {
    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(Clear, area);
    let popup = Paragraph::new(format!(
        "{}\n\nPress Enter or Esc to dismiss",
        error_message
    ))
    .style(Style::default().fg(Color::White).bg(Color::Black))
    .block(
        Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red).bg(Color::Black)),
    );
    frame.render_widget(popup, area);
}

fn render_api_selection_popup(frame: &mut Frame, default_api_state: &mut ListState) {
    let area = centered_rect(60, 25, frame.area());
    let items: Vec<ListItem> = ApiProvider::iter()
        .map(|api| ListItem::new(format!("{:?}", api)))
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title("Select default API")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, default_api_state);
}

fn render_database_reset_popup(frame: &mut Frame, default_reset_state: &mut ListState) {
    let area = centered_rect(60, 25, frame.area());
    let items = vec![
        ListItem::new("Cancel"),
        ListItem::new("Clear transactions and holdings"),
        ListItem::new("Clear everything including tickers"),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .title("Clear database")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, default_reset_state);
}

pub fn render(
    frame: &mut Frame,
    portfolio: &Portfolio,
    table_state: &mut TableState,
    popup_message: &Option<String>,
    error_popup: &Option<String>,
    api_selection_popup: bool,
    default_api_state: &mut ListState,
    selection_mode: bool,
    database_reset_popup: bool,
    default_reset_state: &mut ListState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Table
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_title(frame, portfolio, chunks[0]);
    render_holdings_table(frame, portfolio, table_state, selection_mode, chunks[1]);
    render_footer(frame, chunks[2]);

    if let Some(message) = popup_message {
        render_message_popup(frame, message);
    }

    if let Some(error_message) = error_popup {
        render_error_popup(frame, error_message);
    }

    if api_selection_popup {
        render_api_selection_popup(frame, default_api_state);
    }

    if database_reset_popup {
        render_database_reset_popup(frame, default_reset_state);
    }
}
