//! View rendering for Quirks
//!
//! Handles all terminal UI rendering using ratatui.

use crate::editor::Editor;
use crate::mode::Mode;
use crate::syntax::HighlightSpan;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the editor to the terminal
pub fn render(frame: &mut Frame, editor: &Editor) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Editor area
            Constraint::Length(1), // Status line
            Constraint::Length(1), // Command line
        ])
        .split(frame.area());

    render_editor_area(frame, editor, chunks[0]);
    render_status_line(frame, editor, chunks[1]);
    render_command_line(frame, editor, chunks[2]);

    // Position cursor
    let (cursor_x, cursor_y) = calculate_cursor_position(editor, chunks[0]);
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// Render the main editor area with line numbers and content
fn render_editor_area(frame: &mut Frame, editor: &Editor, area: Rect) {
    let buffer = editor.buffer();
    let scroll_offset = editor.scroll_offset();
    
    // Calculate line number width
    let total_lines = buffer.line_count();
    let line_num_width = total_lines.to_string().len().max(2) as u16;
    
    // Split into line numbers and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_num_width + 1), // Line numbers + separator
            Constraint::Min(1),                      // Content
        ])
        .split(area);

    // Render line numbers
    let mut line_numbers: Vec<Line> = Vec::new();
    for i in 0..area.height as usize {
        let line_idx = scroll_offset + i;
        if line_idx < total_lines {
            line_numbers.push(Line::from(Span::styled(
                format!("{:>width$} ", line_idx + 1, width = line_num_width as usize),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            line_numbers.push(Line::from(Span::styled(
                format!("{:>width$} ", "~", width = line_num_width as usize),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
    let line_num_widget = Paragraph::new(line_numbers);
    frame.render_widget(line_num_widget, chunks[0]);

    // Render content with syntax highlighting
    let highlighter = editor.highlighter();
    let mut content_lines: Vec<Line> = Vec::new();
    for i in 0..area.height as usize {
        let line_idx = scroll_offset + i;
        if line_idx < total_lines {
            let line_content = buffer.line(line_idx);
            let highlights = highlighter.highlight_line(&line_content);
            let spans = apply_highlights(&line_content, &highlights);
            content_lines.push(Line::from(spans));
        } else {
            content_lines.push(Line::from(""));
        }
    }
    let content_widget = Paragraph::new(content_lines);
    frame.render_widget(content_widget, chunks[1]);
}

/// Render the status line
fn render_status_line(frame: &mut Frame, editor: &Editor, area: Rect) {
    let buffer = editor.buffer();
    let cursor = editor.cursor();
    let mode = editor.mode();
    
    // Mode indicator
    let mode_style = match mode {
        Mode::Normal => Style::default().bg(Color::Blue).fg(Color::White),
        Mode::Insert => Style::default().bg(Color::Green).fg(Color::Black),
        Mode::Command => Style::default().bg(Color::Yellow).fg(Color::Black),
    };
    let mode_span = Span::styled(format!(" {} ", mode.display()), mode_style);
    
    // File name
    let file_name = buffer
        .file_name()
        .unwrap_or("[No Name]")
        .to_string();
    let modified = if buffer.is_modified() { " [+]" } else { "" };
    let file_span = Span::styled(
        format!(" {}{} ", file_name, modified),
        Style::default().fg(Color::White),
    );
    
    // Position
    let pos_span = Span::styled(
        format!(" {}:{} ", cursor.line + 1, cursor.col + 1),
        Style::default().fg(Color::DarkGray),
    );
    
    // Build status line
    let left = vec![mode_span, file_span];
    let right = vec![pos_span];
    
    let status = Line::from(left);
    let status_widget = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status_widget, area);
    
    // Right-aligned position (render separately)
    let right_status = Line::from(right);
    let right_widget = Paragraph::new(right_status)
        .style(Style::default().bg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right_widget, area);
}

/// Render the command line (for : commands and messages)
fn render_command_line(frame: &mut Frame, editor: &Editor, area: Rect) {
    let content = if editor.mode() == Mode::Command {
        format!(":{}", editor.command_buffer())
    } else if let Some(msg) = editor.message() {
        msg.to_string()
    } else {
        String::new()
    };
    
    let widget = Paragraph::new(content);
    frame.render_widget(widget, area);
}

/// Apply syntax highlighting spans to a line of text
fn apply_highlights(line: &str, highlights: &[HighlightSpan]) -> Vec<Span<'static>> {
    if highlights.is_empty() {
        return vec![Span::raw(line.to_string())];
    }

    let mut spans = Vec::new();
    let mut last_end = 0;
    let chars: Vec<char> = line.chars().collect();

    for hl in highlights {
        // Add unhighlighted text before this span
        if hl.start > last_end {
            let text: String = chars[last_end..hl.start].iter().collect();
            spans.push(Span::raw(text));
        }
        
        // Add highlighted span
        let text: String = chars[hl.start..hl.end.min(chars.len())].iter().collect();
        spans.push(Span::styled(text, hl.style));
        last_end = hl.end;
    }

    // Add remaining unhighlighted text
    if last_end < chars.len() {
        let text: String = chars[last_end..].iter().collect();
        spans.push(Span::raw(text));
    }

    spans
}

/// Calculate the screen position for the cursor
fn calculate_cursor_position(editor: &Editor, editor_area: Rect) -> (u16, u16) {
    let cursor = editor.cursor();
    let scroll_offset = editor.scroll_offset();
    let buffer = editor.buffer();
    
    // Calculate line number width
    let total_lines = buffer.line_count();
    let line_num_width = total_lines.to_string().len().max(2) as u16 + 1;
    
    let screen_line = cursor.line.saturating_sub(scroll_offset) as u16;
    let screen_col = cursor.col as u16 + line_num_width;
    
    (
        editor_area.x + screen_col.min(editor_area.width - 1),
        editor_area.y + screen_line.min(editor_area.height - 1),
    )
}
