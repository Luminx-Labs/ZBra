#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "xtask reports command progress directly to the terminal"
)]

use std::fmt::Display;
use std::io::IsTerminal as _;

#[derive(Clone, Copy, Debug)]
enum Color {
    Cyan,
    Green,
    Yellow,
    Red,
}

fn paint_stdout(value: impl Display, color: Color) -> String {
    paint(value, color, std::io::stdout().is_terminal())
}


fn paint_stderr(value: impl Display, color: Color) -> String {
    paint(value, color, std::io::stderr().is_terminal())
}

fn paint_status(value: impl Display, color: Color) -> String {
    paint_stdout(value, color)
}

fn paint(value: impl Display, color: Color, is_term: bool) -> String {
    let no_color = std::env::var_os("NO_COLOR").is_some();
    if !is_term || no_color {
        return value.to_string();
    }
    let code = match color {
        Color::Cyan => "\x1b[36m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Red => "\x1b[31m",
    };
    format!("{code}{value}\x1b[0m")
}

macro_rules! info {
  ($($argument:tt)*) => {
      println!("{}", crate::paint_stdout(format_args!($($argument)*), crate::Color::Cyan));
  };
}

macro_rules! success {
  ($($argument:tt)*) => {
      println!("{}", crate::paint_stdout(format_args!($($argument)*), crate::Color::Green));
  };
}

macro_rules! warn {
  ($($argument:tt)*) => {
      eprintln!("{}", crate::paint_stderr(format_args!($($argument)*), crate::Color::Yellow));
  };
}

macro_rules! error {
  ($($argument:tt)*) => {
      eprintln!("{}", crate::paint_stderr(format_args!($($argument)*), crate::Color::Red));
  };
}