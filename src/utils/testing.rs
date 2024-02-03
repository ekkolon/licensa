// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use tempfile::{tempdir, TempDir};

pub fn create_temp_file<N: AsRef<Path>>(name: N) -> (TempDir, PathBuf) {
    let tmp_dir = tempdir().unwrap();
    let tmp_file = &tmp_dir.path().to_path_buf();
    let tmp_file = tmp_file.join::<PathBuf>(name.as_ref().into());
    File::create(&tmp_file).unwrap();
    (tmp_dir, tmp_file.to_owned())
}
