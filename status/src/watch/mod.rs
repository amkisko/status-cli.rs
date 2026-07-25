//! Interactive watchlist dashboard (`status watch`).

mod model;
mod row;
mod ui;

use crate::commands::resolve_targets;
use crate::exit::AppExit;
use model::WatchModel;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct WatchOptions<'a> {
    pub target: Option<&'a str>,
    pub from: Option<&'a Path>,
    pub interval_secs: u64,
    pub max_length: usize,
    pub timeout: Option<u64>,
}

pub fn run_watch(options: WatchOptions<'_>) -> Result<(), AppExit> {
    let targets = resolve_targets(options.target, options.from)?;
    let mut model = WatchModel::new(
        targets,
        options.interval_secs,
        options.max_length,
        options.timeout,
    )
    .map_err(|message| {
        eprintln!("error: {message}");
        AppExit::General
    })?;

    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut model);
    ratatui::restore();
    result.map_err(|error| {
        eprintln!("error: {error}");
        AppExit::General
    })
}

fn run_loop(terminal: &mut DefaultTerminal, model: &mut WatchModel) -> io::Result<()> {
    let tick = Duration::from_millis(100);
    loop {
        let now = Instant::now();
        if model.due_for_refresh(now) {
            model.request_refresh();
        }
        if model.refresh_index.is_some() {
            model.tick_refresh();
        }

        terminal.draw(|frame| ui::draw(frame, model))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && handle_key(model, key.code) {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn handle_key(model: &mut WatchModel, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('r') => {
            model.request_refresh();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            model.select_next();
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            model.select_previous();
            false
        }
        _ => false,
    }
}
