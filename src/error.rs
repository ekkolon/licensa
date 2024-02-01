// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

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
