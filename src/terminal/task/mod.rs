// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod step;

pub use step::*;

use super::{Error, Result};

enum Template {
    Message,
    SpinnerMessage,
}

impl Template {
    pub fn style(&self) -> Result<ProgressStyle> {
        match self {
            Template::SpinnerMessage => {
                Ok(ProgressStyle::default_spinner().template("{spinner} {msg}")?)
            }
            Template::Message => Ok(ProgressStyle::default_spinner().template("{msg}")?),
        }
    }
}

#[derive(PartialEq)]
enum Status {
    Success,
    Running,
    Failed,
    //Pending,
}

impl Status {
    pub fn symbol(&self) -> String {
        match *self {
            //Self::Pending => format!("{}", "✔".bold().dimmed()),
            Self::Success => format!("{}", "✔".bold().green()),
            Self::Failed => format!("{}", "✖".bold().red()),
            _ => "".into(),
        }
    }

    pub fn style(&self) -> Result<ProgressStyle> {
        match self {
            Status::Running => Template::Message.style(),
            _ => Template::SpinnerMessage.style(),
        }
    }
}

/// Represents a task step with a progress spinner and status.
#[derive(Debug, Clone)]
pub struct Task {
    progress_bar: ProgressBar,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    name: String,
    done: bool,
    running: bool,
    indent: usize,
    parent: Option<Rc<Task>>,
}

impl Task {
    /// Creates a new `ProgressTracker`.
    pub fn new<M: AsRef<str>>(message: M) -> Self {
        let mut progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_tab_width(0);

        Task {
            name: message.as_ref().into(),
            done: false,
            running: false,
            progress_bar,
            start_time: None,
            end_time: None,
            indent: 0,
            parent: None,
        }
    }

    /// Creates a new `ProgressTracker`.
    pub fn new_from_parent<M: AsRef<str>>(parent: Task, message: M) -> Self {
        let progress_bar = ProgressBar::new(0);
        progress_bar.enable_steady_tick(Duration::from_millis(100));

        Task {
            parent: Some(Rc::new(parent)),
            ..Task::new(message)
        }
    }

    pub fn with_indent(&mut self, indent: usize) {
        self.indent = indent;
    }

    pub fn finish_ok(&mut self) -> Result<()> {
        self.finish()?;
        self.progress_bar.finish_and_clear();
        self.set_status(Status::Success)?;
        Ok(())
    }

    pub fn finish_err(&mut self) -> Result<()> {
        self.finish()?;
        self.set_status(Status::Failed)?;
        self.progress_bar.finish_and_clear();
        Ok(())
    }

    pub fn human_duration(&self) -> Result<String> {
        if !self.done {
            return Err(Error::TaskNotFinished {
                task: self.name.clone(),
            });
        }

        // Ensure we have valid start and end times.
        let start = self.start_time.unwrap();
        let end = self.end_time.unwrap();

        // Calculate the duration between start and end.
        let duration = end.duration_since(start);
        let seconds = duration.as_secs_f64(); // Total time in seconds

        // Now format the duration based on the given conditions
        if seconds < 0.5 {
            // If duration is less than 0.5s, show milliseconds
            let millis = duration.as_millis();
            return Ok(format!("{}ms", millis));
        }

        // If duration is greater than or equal to 0.5s, show seconds with one decimal
        if seconds < 60.0 {
            return Ok(format!("{:.1}s", seconds));
        }

        // If duration is over a minute, show minutes and seconds (mm:ss)
        let minutes = (seconds / 60.0).floor();
        let remaining_seconds = seconds - (minutes * 60.0);
        Ok(format!("{:02}m {:.0}s", minutes, remaining_seconds))
    }

    fn set_status(&self, status: Status) -> Result<()> {
        self.progress_bar.set_style(status.style()?);

        let message = match &status {
            Status::Running => self.message().to_string(),
            s => format!("{} {}", s.symbol(), &self.message()),
        };

        self.progress_bar.set_message(format!("{}", message.bold()));

        Ok(())
    }

    pub fn progess_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }

    pub fn indent(&self) -> usize {
        match &self.parent {
            None => self.indent,
            Some(parent) => parent.indent + self.indent,
        }
    }

    pub fn logln<M: AsRef<str>>(&mut self, msg: M) -> &Self {
        self.progress_bar.println(format!(
            "{:>indent$}{}",
            "",
            msg.as_ref(),
            indent = self.indent()
        ));
        self
    }

    pub fn print<M: AsRef<str>>(&self, msg: M) -> &Self {
        print!("{:indent$}{}", "", msg.as_ref(), indent = self.indent());
        self
    }

    pub fn line_break(&self) {
        self.print("\n");
    }
}

impl Step for Task {
    /// Check if the task step is finished.
    fn done(&self) -> bool {
        self.done
    }

    fn pending(&self) -> bool {
        !self.done && !self.running
    }

    fn running(&self) -> bool {
        self.running
    }

    /// Get the message of the task step.
    fn message(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn start(&mut self) -> Result<()> {
        let task = self.name.clone();
        if self.done {
            return Err(Error::TaskAlreadyFinished { task });
        }
        if self.running {
            return Err(Error::TaskAlreadyRunning { task });
        }
        self.start_time = Some(Instant::now());
        self.running = true;
        self.set_status(Status::Running)?;
        Ok(())
    }

    /// Mark the task step as finished.
    fn finish(&mut self) -> Result<()> {
        let task = self.name.clone();
        if self.done {
            return Err(Error::TaskAlreadyFinished { task });
        }
        if !self.running {
            return Err(Error::TaskNotRunning { task });
        }

        self.end_time = Some(Instant::now());
        self.running = false;
        self.done = true;
        Ok(())
    }
}
