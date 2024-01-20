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

mod template;

pub use template::*;

// use reqwest::Client;
// use std::{
//   fs,
//   io::{self},
//   path::PathBuf,
// };

// use crate::env::templates_dir;

// #[derive(Debug, Clone)]
// pub struct SpdxTemplateRef {
//   pub spdx_id: String,
// }

// impl TemplateRef for SpdxTemplateRef {
//   fn spdx_id(&self) -> String {
//     self.spdx_id.clone()
//   }
// }

// impl<'a> TemplateRef for Template {
//   fn spdx_id(&self) -> String {
//     self.spdx_id.clone()
//   }
// }

// #[derive(Debug, Clone)]
// pub struct RemoteTemplateSnapshot {
//   content: Vec<u8>,
//   spdx_id: String,
//   title: String,
//   nickname: Option<String>,
// }

// impl TemplateRef for RemoteTemplateSnapshot {
//   fn spdx_id(&self) -> String {
//     self.spdx_id.clone()
//   }
// }

// #[derive(Debug, Clone)]
// pub struct LocalTemplateSnapshot {
//   content: Vec<u8>,
//   spdx_id: String,
//   title: String,
//   nickname: Option<String>,
// }

// impl TemplateRef for LocalTemplateSnapshot {
//   fn spdx_id(&self) -> String {
//     self.spdx_id.clone()
//   }
// }

// impl RemoteTemplateSnapshot {
//   /// Save provided content for this template ref.
//   pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
//     let template_path = &self.path();
//     // Create templates directory if it doesn't exist
//     fs::create_dir_all(templates_dir())?;
//     let content = self.text()?;
//     fs::write(template_path.as_path(), content).unwrap_or_else(|err| {
//       panic!(
//         "\nFailed to write License template for the {} license: {}",
//         &self.spdx_id, err
//       )
//     });
//     println!(
//       "\nSuccessfully saved \"{}\" license",
//       &template_path.display()
//     );

//     Ok(())
//   }
// }
