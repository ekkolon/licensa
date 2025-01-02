use core::fmt;
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{StderrLock, Write},
    time::Instant,
};

use crate::Result;
use colored::{Color, Colorize};

pub struct Logger<'a> {
    handle: std::io::BufWriter<StderrLock<'a>>,
}

impl<'a> Logger<'a> {
    pub fn init() -> Self {
        let writer = std::io::stderr();
        let handle = std::io::BufWriter::new(writer.lock());
        Self { handle }
    }

    /// Write a formatted line to the console
    pub fn write_line(&mut self, line: Line) -> Result<()> {
        writeln!(self.handle, "{}", line.format())?;
        Ok(())
    }

    /// Write multiple formatted lines to the console
    pub fn write_lines<'b, I>(&mut self, lines: I) -> Result<()>
    where
        I: IntoIterator<Item = Line<'b>>,
    {
        for line in lines {
            writeln!(self.handle, "{}", line.format())?;
        }
        Ok(())
    }

    pub fn line_break(&mut self) -> Result<()> {
        writeln!(self.handle)?;
        Ok(())
    }

    pub fn display<M: AsRef<str>>(&mut self, message: M) -> Result<()> {
        writeln!(self.handle, "{}", message.as_ref())?;
        Ok(())
    }

    pub fn human_duration(&self, start: Instant) -> String {
        let end = Instant::now();
        let duration = end.duration_since(start);
        let seconds = duration.as_secs_f64(); // Total time in seconds

        if seconds < 0.5 {
            // If duration is less than 0.5s, show milliseconds
            let millis = duration.as_millis();
            return format!("{}ms", millis);
        }

        if seconds < 60.0 {
            // If duration is greater than or equal to 0.5s, show seconds with one decimal
            return format!("{:.1}s", seconds);
        }

        // If duration is over a minute, show minutes and seconds (mm:ss)
        let minutes = (seconds / 60.0).floor();
        let remaining_seconds = seconds - (minutes * 60.0);
        format!("{:02}m {:.0}s", minutes, remaining_seconds)
    }
}

pub struct Line<'a> {
    content: Cow<'a, str>,
    bindings: HashMap<String, String>,
}

impl<'a> Line<'a> {
    /// Create a new line with the given content template
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
            bindings: HashMap::new(),
        }
    }

    pub fn colored(mut self, color: impl Into<Color>) -> Self {
        let indented_content = format!("{}", self.content.color(color));
        self.content = Cow::from(indented_content);
        self
    }

    /// Bind a key-value pair for interpolation
    pub fn bind(mut self, key: &str, value: impl fmt::Display) -> Self {
        self.bindings.insert(key.to_string(), value.to_string());
        self
    }

    pub fn indent(mut self, spaces: usize) -> Self {
        let indented_content = format!("{:>indent$}{}", "", self.content, indent = spaces);
        self.content = Cow::Owned(indented_content);
        self
    }

    /// Print the interpolated line
    /// Interpolate and write the line to the given handle
    fn format(self) -> String {
        let mut output = self.content.into_owned();
        for (key, value) in self.bindings {
            let placeholder = format!("{{{}}}", key);
            output = output.replace(&placeholder, &value);
        }

        output
    }
}
