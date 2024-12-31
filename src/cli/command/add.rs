// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::console::{Console, Indent};
use crate::io::tree::Tree;
use crate::license::template::copyright::SPDX_COPYRIGHT_NOTICE;
use crate::terminal::Step;
use crate::workspace::{Config, LicensaWorkspace};
use crate::{console, terminal, Error};
use std::io::Write;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::Serialize;

#[derive(Parser, Debug, Serialize, Clone)]
pub struct AddArgs {
    #[command(flatten)]
    config: Config,

    #[arg(short = 'n', long, verbatim_doc_comment)]
    #[arg(default_value_t = true)]
    dry_run: bool,
}

pub fn run(args: &AddArgs) -> Result<()> {
    let mut console = Console::init();

    if args.dry_run {
        console!(
            console,
            "Running in --dry-run mode. No changes will be applied"
        )?;

        console.line_break()?;
    }

    //let mut terminal = terminal::Task::new("Add SPDX license headers");
    //terminal.start()?;

    let mut config = args.config.clone();

    let src_root = std::env::current_dir()?;
    let merged_config = config.merge_into_existing_at_path(&src_root);
    if let Err(err) = merged_config {
        //terminal.finish_err()?;
        err.exit()
    };

    let config = merged_config.unwrap();

    // Verify required fields such es `license`, `owner` and `format` are set.
    if config.license.is_none() {
        crate::Error::MissingRequiredArgument("-t, --type <LICENSE>").exit()
    }

    if config.owner.is_none() {
        crate::Error::MissingRequiredArgument("-o, --owner <OWNER>").exit()
    }

    let config = serde_json::to_value(config);
    if let Err(err) = config {
        crate::Error::ArgumentSerializationFailed {
            arg: "add",
            reason: err.to_string(),
        }
        .exit()
    }

    let config = serde_json::from_value::<LicensaWorkspace>(config.unwrap());
    if let Err(err) = config {
        crate::Error::ArgumentDeserializationFailed {
            arg: "add",
            reason: err.to_string(),
        }
        .exit()
    }

    let config = config.unwrap();

    let template_engine = handlebars::Handlebars::new();
    let template = template_engine.render_template(SPDX_COPYRIGHT_NOTICE, &config)?;

    let mut tree = Tree::new(&src_root);
    tree.set_dry_run(args.dry_run);

    let exclude = Some(config.exclude.clone());
    let entries = tree.find_license_candidates(exclude)?;
    let mut modified_entries = tree.add_license(template, &entries);

    crate::io::utils::sort_paths(&mut modified_entries);

    if args.dry_run {
        console!(
            console,
            "Pending changes for {} files:",
            modified_entries.len()
        )?;

        let suggest_add =
            "(use \"licensa add <glob...>\" without \"--dry-run\" to apply changes)".indent(2);

        console!(console, "{}", suggest_add)?;
        console.log_changes(modified_entries, "modified".indent(8), src_root);
    } else {
        console!(
            console,
            "Added license info to {} files:",
            modified_entries.len()
        )?;
        console.log_changes(modified_entries, "modified".indent(6), src_root);
    }

    if args.dry_run {
        console.line_break()?;
        console!(console, "No changes applied")?;
    }

    Ok(())
}
