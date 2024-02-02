// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use colored::Colorize;

use std::{fmt, time::Instant};

pub struct WorkTreeRunnerStatistics {
    ignored: usize,
    action_count: usize,
    action: String,
    failed: usize,
    num_items: usize,
    start_time: Instant,
    namespace: String,
    status: WorkTreeRunnerStatus,
}

impl WorkTreeRunnerStatistics {
    pub fn new<N>(namespace: N, action: N) -> Self
    where
        N: AsRef<str>,
    {
        Self {
            failed: 0,
            ignored: 0,
            num_items: 0,
            action_count: 0,
            action: action.as_ref().to_string(),
            start_time: Instant::now(),
            namespace: namespace.as_ref().to_string(),
            status: WorkTreeRunnerStatus::Running,
        }
    }

    pub fn add_ignore(&mut self) -> &Self {
        self.ignored += 1;
        self
    }
    pub fn add_action_count(&mut self) -> &Self {
        self.action_count += 1;
        self
    }
    pub fn add_fail(&mut self) -> &Self {
        self.failed += 1;
        self
    }
    pub fn set_items(&mut self, num_items: usize) -> &Self {
        self.num_items = num_items;
        self
    }
    pub fn set_status(&mut self, status: WorkTreeRunnerStatus) -> &Self {
        self.status = status;
        self
    }

    pub fn count_ignored(self) -> usize {
        self.ignored
    }
    pub fn count_passed(self) -> usize {
        self.action_count
    }
    pub fn count_failed(&mut self) -> usize {
        self.failed
    }
    pub fn num_items(&self) -> usize {
        self.num_items
    }
    pub fn status(&self) -> WorkTreeRunnerStatus {
        self.status.clone()
    }

    pub fn elapsed_time(&self) -> String {
        let secs = self.start_time.elapsed().as_secs_f32();
        let mut secs_rounded = secs * 100.0;
        secs_rounded = f32::floor(secs_rounded);
        secs_rounded /= 100.0;
        format!("{secs_rounded}s")
    }

    #[inline]
    pub fn print(&self, line_break: bool) {
        if line_break {
            return println!("\n{}", self);
        }
        println!("{}", self)
    }
}

impl fmt::Display for WorkTreeRunnerStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = format!("{} result: {}", self.namespace, self.status);
        let action = format!("{} {}", self.action_count, self.action);
        let failed = format!("{} failed", self.failed);
        let ignored = format!("{} ignored", self.ignored);
        let duration = format!("finished in {}", self.elapsed_time());
        write!(f, "{status}. {action}; {failed}; {ignored}; {duration}")
    }
}

#[derive(Default, Clone)]
pub enum WorkTreeRunnerStatus {
    Ok,

    #[default]
    Running,

    Failed,
}

impl fmt::Display for WorkTreeRunnerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = self.colorize();
        write!(f, "{status}")
    }
}

impl WorkTreeRunnerStatus {
    pub fn colorize(&self) -> String {
        match *self {
            Self::Failed => "failed".red().to_string(),
            Self::Running => "running".cyan().to_string(),
            Self::Ok => "ok".green().to_string(),
        }
    }
}
