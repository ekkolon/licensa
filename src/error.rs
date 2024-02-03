// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Cli;
use clap::CommandFactory;

pub fn missing_required_arg_error<T>(arg: T) -> !
where
    T: AsRef<str>,
{
    Cli::command()
        .error(
            clap::error::ErrorKind::MissingRequiredArgument,
            format!("Missing required argument {}", arg.as_ref()),
        )
        .exit()
}

pub fn deserialize_args_error(cmd: &str, err: &serde_json::Error) -> ! {
    let err_msg = format!("Failed to deserialize `{cmd}` command arguemnts.\n {}", err);
    Cli::command()
        .error(clap::error::ErrorKind::ValueValidation, err_msg)
        .exit()
}

pub fn serialize_args_error(cmd: &str, err: &serde_json::Error) -> ! {
    let err_msg = format!("Failed to serialize `{cmd}` command arguemnts.\n {}", err);
    Cli::command()
        .error(clap::error::ErrorKind::ValueValidation, err_msg)
        .exit()
}

pub fn exit_invalid_value_err<T>(field: T, value: T, expected: Option<T>)
where
    T: AsRef<str>,
{
    let base_msg = format!(
        "Invalid value {} for field {}.",
        value.as_ref(),
        field.as_ref(),
    );

    let mut cmd = Cli::command();
    if let Some(expected) = expected {
        let msg = format!("{base_msg} {}", expected.as_ref());
        cmd.error(clap::error::ErrorKind::InvalidValue, msg).exit()
    }

    cmd.error(clap::error::ErrorKind::InvalidValue, base_msg)
        .exit()
}

pub fn exit_io_error<M>(err: M)
where
    M: std::fmt::Display,
{
    Cli::command().error(clap::error::ErrorKind::Io, err).exit();
}
