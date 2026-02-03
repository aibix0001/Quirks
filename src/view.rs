//! View rendering for Quirks
//!
//! Handles all terminal UI rendering using ratatui.

use crate::editor::Editor;
use crate::mode::Mode;
use crate::search::{SearchDirection, SearchMatch};
use crate::selection::Selection;
use crate::syntax::HighlightSpan;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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

    // Render help overlay if in help mode
    if editor.mode() == Mode::Help {
        render_help_overlay(frame);
    }

    // Position cursor
    let (cursor_x, cursor_y) = calculate_cursor_position(editor, chunks[0]);
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// Render the main editor area with line numbers and content
fn render_editor_area(frame: &mut Frame, editor: &Editor, area: Rect) {
    let buffer = editor.buffer();
    let scroll_offset = editor.scroll_offset();
    let show_line_numbers = editor.config().line_numbers;

    // Calculate line number width (only if showing)
    let total_lines = buffer.line_count();
    let line_num_width = if show_line_numbers {
        total_lines.to_string().len().max(2) as u16
    } else {
        0
    };

    // Split into line numbers and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(if show_line_numbers { line_num_width + 1 } else { 0 }),
            Constraint::Min(1),
        ])
        .split(area);

    // Render line numbers (if enabled)
    if show_line_numbers {
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
    }

    // Render content with syntax, search, and selection highlighting
    let highlighter = editor.highlighter();
    let search = editor.search();
    let selection = editor.selection();
    let search_matches: Vec<&SearchMatch> = if search.highlight_active {
        search.matches().iter().collect()
    } else {
        Vec::new()
    };

    let mut content_lines: Vec<Line> = Vec::new();
    for i in 0..area.height as usize {
        let line_idx = scroll_offset + i;
        if line_idx < total_lines {
            let line_content = buffer.line(line_idx);
            let syntax_highlights = highlighter.highlight_line(&line_content);

            // Get search matches for this line
            let line_search_matches: Vec<&SearchMatch> = search_matches
                .iter()
                .filter(|m| m.line == line_idx)
                .copied()
                .collect();

            let spans = apply_all_highlights(&line_content, line_idx, &syntax_highlights, &line_search_matches, selection);
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
        Mode::Search => Style::default().bg(Color::Magenta).fg(Color::White),
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => Style::default().bg(Color::Cyan).fg(Color::Black),
        Mode::Help => Style::default().bg(Color::DarkGray).fg(Color::White),
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

    // Position and percentage
    let line_count = buffer.line_count();
    let percent = if line_count <= 1 {
        "All".to_string()
    } else {
        let pct = ((cursor.line + 1) * 100) / line_count;
        format!("{}%", pct)
    };
    let pos_span = Span::styled(
        format!(" {}:{} {} Ln {} ", cursor.line + 1, cursor.col + 1, percent, line_count),
        Style::default().fg(Color::DarkGray),
    );

    // Syntax indicator
    let syntax_name = editor.highlighter().current_syntax_name().unwrap_or("Plain");
    let syntax_span = Span::styled(
        format!(" {} ", syntax_name),
        Style::default().fg(Color::Cyan),
    );

    // Build status line
    let left = vec![mode_span, file_span];
    let mut right = vec![syntax_span, pos_span];

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
    } else if editor.mode() == Mode::Search {
        let prefix = match editor.search().direction() {
            SearchDirection::Forward => "/",
            SearchDirection::Backward => "?",
        };
        format!("{}{}", prefix, editor.search().pattern())
    } else if let Some(msg) = editor.message() {
        msg.to_string()
    } else {
        String::new()
    };

    let widget = Paragraph::new(content);
    frame.render_widget(widget, area);
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame) {
    let help_text = vec![
        "",
        "  ╔══════════════════════════════════════════════════════════════╗",
        "  ║                      QUIRKS HELP                             ║",
        "  ╠══════════════════════════════════════════════════════════════╣",
        "  ║  NAVIGATION                                                  ║",
        "  ║    h/j/k/l     Move left/down/up/right                       ║",
        "  ║    w/b/e       Word forward/backward/end                     ║",
        "  ║    0/$         Line start/end                                ║",
        "  ║    gg/G        Buffer start/end                              ║",
        "  ║    gt/gT       Next/previous buffer                          ║",
        "  ║                                                              ║",
        "  ║  EDITING                                                     ║",
        "  ║    i/a         Insert before/after cursor                    ║",
        "  ║    I/A         Insert at line start/end                      ║",
        "  ║    o/O         New line below/above                          ║",
        "  ║    x           Delete character                              ║",
        "  ║    dd          Delete line                                   ║",
        "  ║    yy          Yank (copy) line                              ║",
        "  ║    p/P         Paste after/before                            ║",
        "  ║    u/Ctrl+R    Undo/Redo                                     ║",
        "  ║                                                              ║",
        "  ║  SEARCH                                                      ║",
        "  ║    /pattern    Search forward                                ║",
        "  ║    ?pattern    Search backward                               ║",
        "  ║    n/N         Next/previous match                           ║",
        "  ║                                                              ║",
        "  ║  COMMANDS                                                    ║",
        "  ║    :w          Save file     :e <file>   Open file           ║",
        "  ║    :q          Quit          :ls         List buffers        ║",
        "  ║    :wq         Save & quit   :b <N>      Switch buffer       ║",
        "  ║    :q!         Force quit    :bd         Close buffer        ║",
        "  ║    :set        Show config   :help       This help           ║",
        "  ║                                                              ║",
        "  ║  OTHER                                                       ║",
        "  ║    Ctrl+G      File info     Ctrl+Q      Force quit          ║",
        "  ║    >>          Indent        <<          Outdent             ║",
        "  ║                                                              ║",
        "  ║           Press q, Esc, or Enter to close                    ║",
        "  ╚══════════════════════════════════════════════════════════════╝",
        "",
    ];

    let area = frame.area();
    let help_height = help_text.len() as u16;
    let help_width = 68;
    
    let x = area.width.saturating_sub(help_width) / 2;
    let y = area.height.saturating_sub(help_height) / 2;
    
    let help_area = Rect::new(x, y, help_width, help_height);
    
    // Clear background
    let block = ratatui::widgets::Block::default()
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block, help_area);
    
    // Render help text
    let text: Vec<Line> = help_text.iter().map(|s| Line::from(*s)).collect();
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    frame.render_widget(paragraph, help_area);
}

/// Apply all highlighting (syntax, search, selection) to a line of text
fn apply_all_highlights(
    line: &str,
    line_idx: usize,
    syntax_highlights: &[HighlightSpan],
    search_matches: &[&SearchMatch],
    selection: Option<&Selection>,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = line.chars().collect();

    if syntax_highlights.is_empty() && search_matches.is_empty() && selection.is_none() {
        return vec![Span::raw(line.to_string())];
    }

    let mut spans = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        // Check if we're in a selection
        let in_selection = selection.map_or(false, |s| s.contains(line_idx, i));

        // Check if we're in a search match
        let search_match = search_matches.iter().find(|m| i >= m.start_col && i < m.end_col);

        // Check if we're in a syntax highlight
        let syntax_hl = syntax_highlights.iter().find(|h| i >= h.start && i < h.end);

        if in_selection {
            // Selection - find extent
            let mut end = i + 1;
            while end < chars.len() && selection.map_or(false, |s| s.contains(line_idx, end)) {
                end += 1;
            }
            let text: String = chars[i..end].iter().collect();

            // Apply syntax color on selection background
            let fg = syntax_hl.map(|h| h.style.fg.unwrap_or(Color::White)).unwrap_or(Color::White);
            spans.push(Span::styled(
                text,
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(fg),
            ));
            i = end;
        } else if let Some(sm) = search_match {
            // Search match takes priority - render with highlight background
            let end = sm.end_col.min(chars.len());
            let text: String = chars[i..end].iter().collect();
            spans.push(Span::styled(
                text,
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
            i = end;
        } else if let Some(sh) = syntax_hl {
            // Regular syntax highlight
            let end = sh.end.min(chars.len());
            // Check if selection or search starts before syntax ends
            let selection_start = if selection.is_some() {
                (i+1..end).find(|&c| selection.map_or(false, |s| s.contains(line_idx, c)))
            } else {
                None
            };
            let search_start = search_matches
                .iter()
                .filter(|m| m.start_col > i && m.start_col < end)
                .map(|m| m.start_col)
                .min();
            let actual_end = [selection_start, search_start, Some(end)]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(end);

            let text: String = chars[i..actual_end].iter().collect();
            spans.push(Span::styled(text, sh.style));
            i = actual_end;
        } else {
            // No highlight - find where the next highlight starts
            let next_syntax = syntax_highlights
                .iter()
                .filter(|h| h.start > i)
                .map(|h| h.start)
                .min();
            let next_search = search_matches
                .iter()
                .filter(|m| m.start_col > i)
                .map(|m| m.start_col)
                .min();
            let next_selection = if selection.is_some() {
                (i+1..chars.len()).find(|&c| selection.map_or(false, |s| s.contains(line_idx, c)))
            } else {
                None
            };
            let next = [next_syntax, next_search, next_selection]
                .into_iter()
                .flatten()
                .min();

            let end = next.unwrap_or(chars.len()).min(chars.len());
            let text: String = chars[i..end].iter().collect();
            spans.push(Span::raw(text));
            i = end;
        }
    }

    // Handle empty line with selection
    if chars.is_empty() && selection.map_or(false, |s| s.contains(line_idx, 0)) {
        spans.push(Span::styled(" ", Style::default().bg(Color::DarkGray)));
    }

    if spans.is_empty() {
        spans.push(Span::raw(String::new()));
    }

    spans
}

/// Calculate the screen position for the cursor
fn calculate_cursor_position(editor: &Editor, editor_area: Rect) -> (u16, u16) {
    let cursor = editor.cursor();
    let scroll_offset = editor.scroll_offset();
    let buffer = editor.buffer();
    let show_line_numbers = editor.config().line_numbers;

    // Calculate line number width (only if showing)
    let line_num_width = if show_line_numbers {
        let total_lines = buffer.line_count();
        total_lines.to_string().len().max(2) as u16 + 1
    } else {
        0
    };

    let screen_line = cursor.line.saturating_sub(scroll_offset) as u16;
    let screen_col = cursor.col as u16 + line_num_width;

    (
        editor_area.x + screen_col.min(editor_area.width - 1),
        editor_area.y + screen_line.min(editor_area.height - 1),
    )
}
