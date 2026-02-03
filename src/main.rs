//! Quirks - A modal text editor with character
//!
//! Born from Vim's efficiency and Emacs' extensibility.

mod buffer;

use buffer::Buffer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Load file if provided, otherwise create empty buffer
    let buffer = if args.len() > 1 {
        match Buffer::from_file(&args[1]) {
            Ok(buf) => {
                eprintln!("Loaded: {} ({} lines)", args[1], buf.line_count());
                buf
            }
            Err(e) => {
                eprintln!("Error loading file: {}", e);
                Buffer::new()
            }
        }
    } else {
        eprintln!("Quirks v0.1.0 - No file specified");
        Buffer::new()
    };

    // TODO: Initialize TUI
    // TODO: Enter event loop
    // TODO: Render buffer content
    
    // For now, just print buffer info
    println!("Buffer: {} chars, {} lines", 
             buffer.char_count(), 
             buffer.line_count());
    
    if let Some(path) = buffer.filepath() {
        println!("File: {}", path.display());
    }

    // Print first 10 lines as preview
    println!("\n--- Preview ---");
    for i in 0..buffer.line_count().min(10) {
        if let Some(line) = buffer.line(i) {
            print!("{:4} │ {}", i + 1, line);
        }
    }
    if buffer.line_count() > 10 {
        println!("... ({} more lines)", buffer.line_count() - 10);
    }
}
