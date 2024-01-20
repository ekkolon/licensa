// Copyright 2024 Nelson Dominguez
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Licensa configuration file parser and utils

use std::{env::current_dir, fs};

use serde::{Deserialize, Serialize};

use crate::utils;

const DEFAULT_LICENSE_TYPE: &str = "MIT";
const CONFIG_FILE_NAME: &str = ".licensarc";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_license_type", rename = "type")]
  pub license_type: String,

  pub author: String,

  #[serde(default = "utils::current_year")]
  pub year: u16,

  pub generator: GeneratorConfig,
}

impl Config {
  pub fn parse() -> Result<Config, Box<dyn std::error::Error>> {
    let config_file_path = current_dir()?.join(CONFIG_FILE_NAME);
    let content = fs::read_to_string(config_file_path)?;
    let config = serde_json::from_str::<Config>(&content)?;
    Ok(config)
  }
}

fn default_license_type() -> String {
  DEFAULT_LICENSE_TYPE.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorConfig {
  #[serde(default = "default_spdx_only_flag")]
  pub spdx_only: bool,

  pub license_file: Option<String>,

  #[serde(default = "default_allowed_licenses")]
  pub allowed_licenses: Vec<String>,

  #[serde(default = "default_ignore_patterns")]
  pub ignore_patterns: Vec<String>,

  #[serde(default = "default_gitignore")]
  pub gitignore: bool,
}

fn default_ignore_patterns<'a>() -> Vec<String> {
  vec![]
}

fn default_gitignore() -> bool {
  true
}

fn default_spdx_only_flag() -> bool {
  false
}

fn default_allowed_licenses<'a>() -> Vec<String> {
  vec!["*".to_string()]
}
