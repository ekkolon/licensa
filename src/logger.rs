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

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn init() {
  Builder::new()
    .format(|buf, record| {
      writeln!(
        buf,
        "{} [{}] - {}",
        Local::now().format("%Y-%m-%dT%H:%M:%S"),
        record.level(),
        record.args()
      )
    })
    .filter(None, LevelFilter::Info)
    .init();
}
