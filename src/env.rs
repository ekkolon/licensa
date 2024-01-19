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

use std::path::PathBuf;

use directories::ProjectDirs;

const PROJECT_NAME: &str = "Licensa";
const LOCALE_TEMPLATES_DIR_NAME: &str = "templates";

// TODO: Add docs
// TODO: Add test
pub fn templates_dir() -> PathBuf {
  data_dir().join(LOCALE_TEMPLATES_DIR_NAME)
}

// TODO: Add docs
// TODO: Add test
pub fn data_dir() -> PathBuf {
  project_dirs().data_dir().to_path_buf()
}

// TODO: Add docs
// TODO: Add test
fn project_dirs() -> ProjectDirs {
  ProjectDirs::from("", "", PROJECT_NAME).expect("Failed to determine app data directory")
}
