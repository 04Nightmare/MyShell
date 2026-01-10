use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Write};

use crate::auto_complete;
use crate::terminal::draw_line::redraw_entire_line;
use crate::utilities::executable::print_all_exec;
use crate::utilities::history::fetch_history_commands;
use crate::utilities::longest_prefix::longest_common_prefix;
use crate::utilities::redirect::handle_redirect_append;

pub fn read_inputs_keypress() -> String {
    enable_raw_mode().unwrap();
    let mut buffer = String::new();
    let mut tab_flip = false;
    let mut history_index = 0;
    loop {
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key {
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('m'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    print!("\r\n");
                    if !buffer.is_empty() {
                        handle_redirect_append(&String::from("history.txt"), buffer.as_bytes());
                    }
                    io::stdout().flush().unwrap();
                    break;
                }

                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    if !buffer.is_empty() {
                        tab_flip = false;
                        buffer.pop();
                        print!("\x1b[D\x1b[K");
                        io::stdout().flush().unwrap();
                    }
                }

                KeyEvent {
                    code: KeyCode::Up, ..
                } => {
                    history_index += 1;
                    if let Some(his_buffer) = fetch_history_commands(history_index) {
                        buffer = his_buffer;
                        redraw_entire_line("$ ", &buffer);
                    } else {
                        history_index -= 1;
                    }
                }
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => {
                    if history_index == 0 {
                        continue;
                    }
                    history_index -= 1;
                    if history_index == 0 {
                        buffer.clear();
                    } else if let Some(his_buffer) = fetch_history_commands(history_index) {
                        buffer = his_buffer;
                    }
                    redraw_entire_line("$ ", &buffer);
                }

                KeyEvent {
                    code: KeyCode::Tab, ..
                } => {
                    if buffer.is_empty() {
                        println!("\x07");
                        redraw_entire_line("$ ", &buffer);
                        tab_flip = false;
                        continue;
                    }
                    let (string_option, string_vector) = auto_complete(&buffer);
                    match string_vector.len() {
                        0 => {
                            println!("\x07");
                            redraw_entire_line("$ ", &buffer);
                            tab_flip = false;
                        }
                        1 => {
                            if let Some(complete_command) = string_option {
                                buffer = complete_command;
                                buffer.push(' ');
                                redraw_entire_line("$ ", &buffer);
                                tab_flip = false;
                            }
                        }
                        _ => {
                            let common = longest_common_prefix(&string_vector);
                            if common.len() > buffer.len() {
                                buffer = common;
                                redraw_entire_line("$ ", &buffer);
                                tab_flip = false;
                            } else {
                                if tab_flip == false {
                                    print!("\x07");
                                    redraw_entire_line("$ ", &buffer);
                                    tab_flip = true;
                                } else {
                                    print!("\r\n");
                                    print_all_exec(string_vector);
                                    redraw_entire_line("$ ", &buffer);
                                    tab_flip = false;
                                }
                            }
                        }
                    }
                }

                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: m,
                    ..
                } if m.is_empty() || m == KeyModifiers::SHIFT => {
                    tab_flip = false;
                    buffer.push(c);
                    print!("{}", c);
                    io::stdout().flush().unwrap();
                }

                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    tab_flip = false;
                    print!("^C\r\n");
                    buffer.clear();
                    redraw_entire_line("$ ", &buffer);
                    history_index = 0;
                }

                _ => {}
            }
        }
    }
    disable_raw_mode().unwrap();
    return buffer;
}
