use crate::span::Span;
use colored::*;
use std::fmt::Display;

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    message: String,
    span: Span,
}

impl Error {
    pub(crate) fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (filename, range) = self.span.clone();
        let (start, end) = (range.start, range.end);
        let (line_number, (start, end)) = (start.0, (start.1, end.1));
        let contents: String = std::fs::read_to_string(filename.as_str()).unwrap();
        let line = contents.lines().nth(line_number - 1).unwrap();

        let mut out = String::new();
        out.push_str(&format!(
            "{}{}{}\n",
            format!("{}:{}:{}: ", filename, line_number, start)
                .white()
                .bold(),
            "error: ".red().bold(),
            self.message.white().bold()
        ));
        out.push_str(&format!("{}\n", line));
        out.push_str(
            format!("{}^{}", " ".repeat(start - 1), "~".repeat(end - start),)
                .green()
                .to_string()
                .as_str(),
        );

        write!(f, "{}", out)
    }
}
