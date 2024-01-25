// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use directories::ProjectDirs;

const PROJECT_NAME: &str = "Licensa";
const LOCALE_TEMPLATES_DIR_NAME: &str = "templates";

// TODO: Add docs
// TODO: Add test
#[inline]
pub fn templates_dir() -> PathBuf {
  data_dir().join(LOCALE_TEMPLATES_DIR_NAME)
}

// TODO: Add docs
// TODO: Add test
#[inline]
pub fn data_dir() -> PathBuf {
  project_dirs().data_dir().to_path_buf()
}

// TODO: Add docs
// TODO: Add test
#[inline]
fn project_dirs() -> ProjectDirs {
  ProjectDirs::from("", "", PROJECT_NAME)
    .expect("Failed to determine app data directory")
}
