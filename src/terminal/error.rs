// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Task is already running:\n{:>4}Task: {task}", "")]
    TaskAlreadyRunning { task: String },

    #[error("Task is already finished:\n{:>4}Task: {task}", "")]
    TaskAlreadyFinished { task: String },

    #[error("Task is not finished:\n{:>4}Task: {task}", "")]
    TaskNotFinished { task: String },

    #[error(
        "Task cannot be finished without being started first:\n{:>4}Task: {task}",
        ""
    )]
    TaskNotRunning { task: String },

    #[error(transparent)]
    Progress(#[from] indicatif::style::TemplateError),
}
