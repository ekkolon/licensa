use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

mod step;

pub use step::*;

use super::{Error, Result};

enum TaskState {
    Done,
    Running,
}

impl TaskState {
    fn print_with(&self) -> std::fmt::Result {
        match *self {
            Self::Done => write!(f, "{} Done", "✔".green()),
            Self::Failed => write!(f, "({}, {})", "✖".red()),
            Self::Running => write!(f, "({}, {})", self.longitude, self.latitude),
        }
    }
}

/// Represents a task step with a progress spinner and status.
pub struct ConsoleStep {
    name: String,
    done: bool,
    running: bool,
    progress_bar: ProgressBar,
}

impl ConsoleStep {
    /// Creates a new `TaskStep`.
    pub fn new<M: AsRef<str>>(progress_bar: ProgressBar, message: M) -> Self {
        Self {
            name: message.as_ref().into(),
            done: false,
            running: true,
            progress_bar,
        }
    }

    /// Creates a new `TaskStep`.
    pub fn lazy<M: AsRef<str>>(progress_bar: ProgressBar, message: M) -> Self {
        Self {
            running: false,
            ..Self::new(progress_bar, message)
        }
    }

    pub fn finish_ok(&mut self) -> Result<()> {
        self.finish()?;
        self.progress_bar
            .println(format!("{} {}", "✔".green(), &self.message().green()));
        Ok(())
    }

    pub fn finish_err(&mut self) -> Result<()> {
        self.finish()?;
        self.progress_bar
            .println(format!("{} {}", "✖".red(), &self.message().red()));
        Ok(())
    }
}

impl Step for ConsoleStep {
    /// Mark the task step as finished.
    fn finish(&mut self) -> Result<()> {
        if self.done {
            return Err(Error::StepAlreadyFinished {
                step: self.name.clone(),
            });
        }
        self.done = true;
        Ok(())
    }

    fn running(&self) -> bool {
        self.running
    }

    /// Get the message of the task step.
    fn message(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    /// Check if the task step is finished.
    fn done(&self) -> bool {
        self.done
    }
}

impl LazyStep for ConsoleStep {
    fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(Error::StepAlreadyRunning {
                step: self.name.clone(),
            });
        }

        self.progress_bar.set_message(self.message().to_string());
        self.running = true;
        Ok(())
    }
}

/// Progress tracker for step-based tasks.
pub struct Task {
    progress_bar: ProgressBar,
    start_time: Instant,
}

impl Task {
    /// Creates a new `ProgressTracker`.
    pub fn new() -> Self {
        let progress_bar = ProgressBar::new(0);
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} msg}")
                .unwrap(),
        );

        Task {
            progress_bar,
            start_time: Instant::now(),
        }
    }

    /// Sets a new step message with a spinner.
    pub fn step<S: AsRef<str>>(&self, message: S) -> ConsoleStep {
        let step = ConsoleStep::new(self.progress_bar.clone(), message.as_ref());
        self.progress_bar.set_message(step.name.to_string());
        step
    }

    /// Marks the current step as finished with a success checkmark.
    pub fn finish_step(&self, message: &str) {
        self.progress_bar
            .println(format!("{} {}", "✔".green(), message.green()));
    }

    /// Finish the progress tracking with a summary message.
    pub fn finish_tracking(&self, message: &str) {
        self.progress_bar.
        self.progress_bar.finish_with_message(message);
        self.print_stats();
    }

    /// Calculate elapsed time since start.
    pub fn elapsed_time(&self) -> String {
        format!("{:.2} seconds", self.start_time.elapsed().as_secs_f64())
    }

    fn set_template() {}
}
