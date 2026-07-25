//! Ratatui drawing for the watch dashboard.

use super::model::WatchModel;
use super::row::RowKind;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw(frame: &mut Frame<'_>, model: &WatchModel) {
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(frame.area());

    draw_header(frame, areas[0], model);
    draw_table(frame, areas[1], model);
    draw_detail(frame, areas[2], model);
    draw_footer(frame, areas[3], model);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, model: &WatchModel) {
    let ok = model
        .rows
        .iter()
        .filter(|row| row.kind == RowKind::Ok)
        .count();
    let degraded = model
        .rows
        .iter()
        .filter(|row| row.kind == RowKind::Degraded || row.kind == RowKind::Error)
        .count();
    let title = format!(
        " status watch · {} targets · {ok} ok · {degraded} attention ",
        model.rows.len()
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);
}

fn draw_table(frame: &mut Frame<'_>, area: Rect, model: &WatchModel) {
    let header = Row::new(vec!["Service", "Status", "URL"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(0);
    let rows = model.rows.iter().enumerate().map(|(index, row)| {
        let style = row_style(row.kind, index == model.selected);
        Row::new(vec![
            truncate(&row.label, 28),
            truncate(&row.status_text, 32),
            truncate(&row.status_url, 48),
        ])
        .style(style)
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(28),
            Constraint::Length(32),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Watchlist "));
    frame.render_widget(table, area);
}

fn draw_detail(frame: &mut Frame<'_>, area: Rect, model: &WatchModel) {
    let detail = model
        .rows
        .get(model.selected)
        .map(|row| {
            if row.detail.is_empty() {
                format!("{} · {}", row.label, row.status_url)
            } else {
                format!("{} — {}", row.label, row.detail)
            }
        })
        .unwrap_or_default();
    let widget = Paragraph::new(detail).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Selected ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(widget, area);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, model: &WatchModel) {
    let line = Line::from(vec![Span::styled(
        model.message.as_str(),
        Style::default().fg(Color::Gray),
    )]);
    frame.render_widget(Paragraph::new(line), area);
}

fn row_style(kind: RowKind, selected: bool) -> Style {
    let mut style = match kind {
        RowKind::Ok => Style::default().fg(Color::Green),
        RowKind::Degraded => Style::default().fg(Color::Yellow),
        RowKind::Error => Style::default().fg(Color::Red),
        RowKind::Loading => Style::default().fg(Color::Cyan),
        RowKind::Pending => Style::default().fg(Color::DarkGray),
    };
    if selected {
        style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
    }
    style
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let shortened: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{shortened}…")
}
