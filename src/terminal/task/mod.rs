use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod step;

pub use step::*;

use super::{Error, Result};

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
        let progress_bar = ProgressBar::new(0);
        progress_bar.enable_steady_tick(Duration::from_millis(100));

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
        self.indent = indent
    }

    /// Creates a new `TaskStep`.
    pub fn lazy<M: AsRef<str>>(message: M) -> Self {
        Self {
            running: false,
            ..Self::new(message)
        }
    }

    pub fn finish_ok(&mut self) -> Result<()> {
        self.finish()?;
        self.print_status(Status::Success)?;
        Ok(())
    }

    pub fn finish_err(&mut self) -> Result<()> {
        self.finish()?;
        self.print_status(Status::Failed)?;
        Ok(())
    }

    /// Calculate elapsed time since start.
    pub fn duration_in_secs(&self) -> Result<String> {
        if !self.done {
            return Err(Error::TaskNotFinished {
                task: self.name.clone(),
            });
        }
        // We can safely unwrap here because when `done` is true start and end time are set.
        let time = self.end_time.unwrap() - self.start_time.unwrap();
        let time_in_secs = format!("{:.2} seconds", time.as_secs_f64());
        Ok(time_in_secs)
    }

    fn print_status(&self, status: Status) -> Result<()> {
        let template = match status {
            Status::Running => ProgressStyle::default_spinner().template("{spinner} {msg}")?,
            _ => ProgressStyle::default_spinner().template("{msg}")?,
        };

        self.progress_bar.set_style(template);

        let message = match &status {
            Status::Running => self.message().to_string(),
            s => format!("{} {}", s.symbol(), &self.message()),
        };

        match status {
            Status::Success | Status::Failed => self
                .progress_bar
                .abandon_with_message(format!("{}", message.bold())),
            _ => self.progress_bar.set_message(format!("{}", message.bold())),
        };

        Ok(())
    }

    pub fn indent(&self) -> usize {
        match &self.parent {
            None => self.indent + 2,
            Some(parent) => parent.indent + self.indent + 2,
        }
    }

    pub fn logln<M: AsRef<str>>(&self, msg: M) -> &Self {
        println!("{:>1$}{2}", "", &self.indent(), msg.as_ref());
        self
    }

    pub fn print<M: AsRef<str>>(&self, msg: M) -> &Self {
        print!("{:>1$}{2}", "", &self.indent(), msg.as_ref());
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
        self.print_status(Status::Running)?;
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
        self.done = true;
        Ok(())
    }
}
