// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_macros)]

macro_rules! notice {
    ($message:expr) => {
        let level = "INFO".blue().bold();
        println!("{level}: {}", $message);
    };
}
pub(crate) use notice;

macro_rules! success {
    ($message:expr) => {
        let level = "SUCCESS".green().bold();
        println!("{level}: {}", $message);
    };
}
pub(crate) use success;

macro_rules! failure {
    ($message:expr) => {
        let level = "FAILURE".red().bold();
        println!("{level}: {}", $message);
    };
}
// pub(crate) use failure;

macro_rules! warning {
    ($message:expr) => {
        let level = "WARNING".yellow().bold();
        println!("{level}: {}", $message);
    };
}
pub(crate) use warning;
