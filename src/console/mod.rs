use core::fmt;
use std::{
    borrow::Cow,
    fmt::Display,
    io::{StderrLock, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{Error, Result};

pub struct Console<'a> {
    handle: std::io::BufWriter<StderrLock<'a>>,
}

impl<'a> Console<'a> {
    pub fn init() -> Self {
        let writer = std::io::stderr();
        let handle = std::io::BufWriter::new(writer.lock());
        Self { handle }
    }

    // A helper method to make `writeln` easy to call
    pub fn writeln_fmt(&mut self, args: fmt::Arguments) -> Result<()> {
        writeln!(self.handle, "{}", args)?;
        Ok(())
    }

    pub fn line_break(&mut self) -> Result<()> {
        write!(self.handle, "\n")?;
        Ok(())
    }

    pub fn writeln<M: AsRef<str>>(&mut self, message: M) -> Result<()> {
        writeln!(self.handle, "{}", message.as_ref())?;
        Ok(())
    }

    pub fn log_changes<K: AsRef<str>, P: AsRef<Path>>(
        &mut self,
        entries: Vec<PathBuf>,
        kind: K,
        strip_prefix: P,
    ) {
        entries
            .iter()
            .filter_map(|path| path.strip_prefix(&strip_prefix).ok())
            .map(|path| format!("{}:  {}", kind.as_ref().bold(), path.display()))
            .for_each(|message| {
                let write_op = writeln!(self.handle, "{}", message.green());
                if let Err(err) = write_op {
                    Error::Io(err).exit();
                };
            });
    }
}

pub trait Indent {
    fn indent(&self, indent: usize) -> Cow<'_, str>;
}

impl<T: AsRef<str> + Display> Indent for T {
    fn indent(&self, indent: usize) -> Cow<'_, str> {
        Cow::Owned(format!("{:>indent$}{}", "", self, indent = indent))
    }
}

#[macro_use]
mod macros {
    // Updated macro to call `writeln_fmt` with `fmt::Arguments`
    #[macro_export]
    macro_rules! console {
        ($console:expr, $($arg:tt)*) => {
            $console.writeln_fmt(format_args!($($arg)*))
        };
    }
}
