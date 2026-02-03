//! Quirks - A modern text editor
//!
//! Born from the union of Vim's modal efficiency and Emacs' extensibility.
//! Created by Egon and Aibotix.

mod buffer;
mod cursor;
mod buffer_manager;
mod editor;
mod history;
mod mode;
mod register;
mod search;
mod selection;
mod syntax;
mod view;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io};

fn main() -> Result<()> {
    // Get file argument if provided
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).map(|s| s.as_str());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create editor
    let mut editor = editor::Editor::new();
    if let Some(path) = file_path {
        editor.open_file(path)?;
    }

    // Main loop
    let result = run_editor(&mut terminal, &mut editor);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    editor: &mut editor::Editor,
) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|frame| {
            view::render(frame, editor);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            // Ctrl+Q to quit (always)
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                break;
            }

            // Pass to editor
            if editor.handle_key(key) {
                break;
            }
        }
    }
    Ok(())
}
