#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "xtask reports command progress directly to the terminal"
)]

use std::fmt::Display;
use std::io::IsTerminal as _;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Color {
    Cyan,
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Level {
    Info,
    Success,
    Warn,
    Error,
}

impl Level {
    const fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Success => "SUCCESS",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Info => Color::Cyan,
            Self::Success => Color::Green,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
        }
    }
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

fn format_message(level: Level, message: impl Display, is_term: bool) -> String {
    format!(
        "{}  {message}",
        paint(level.label(), level.color(), is_term)
    )
}

pub(crate) fn emit_stdout(level: Level, message: impl Display) {
    println!(
        "{}",
        format_message(level, message, std::io::stdout().is_terminal())
    );
}

pub(crate) fn emit_stderr(level: Level, message: impl Display) {
    eprintln!(
        "{}",
        format_message(level, message, std::io::stderr().is_terminal())
    );
}

#[macro_export]
macro_rules! info {
  ($($argument:tt)*) => {
      $crate::log::emit_stdout($crate::log::Level::Info, format_args!($($argument)*))
  };
}

#[macro_export]
macro_rules! success {
  ($($argument:tt)*) => {
      $crate::log::emit_stdout($crate::log::Level::Success, format_args!($($argument)*))
  };
}

#[macro_export]
macro_rules! warn {
  ($($argument:tt)*) => {
      $crate::log::emit_stderr($crate::log::Level::Warn, format_args!($($argument)*))
  };
}

#[macro_export]
macro_rules! error {
  ($($argument:tt)*) => {
      $crate::log::emit_stderr($crate::log::Level::Error, format_args!($($argument)*))
  };
}

#[cfg(test)]
mod tests {
    use super::{Level, format_message};

    #[test]
    fn formats_level_before_message() {
        assert_eq!(
            format_message(Level::Info, "checking workspace", false),
            "INFO  checking workspace"
        );
        assert_eq!(
            format_message(Level::Success, "done", false),
            "SUCCESS  done"
        );
    }
}
