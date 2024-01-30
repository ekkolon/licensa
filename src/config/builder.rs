// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

use super::{Config, CopyrightNoticeFormat};

// TODO: Add docs
// TODO: Add tests
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        self.config
    }

    // TODO: Validate SPDX id
    pub fn license<T>(&mut self, license: T) -> &Self
    where
        T: AsRef<str>,
    {
        self.config.license = license.as_ref().into();
        self
    }

    pub fn holder<T>(&mut self, holder: T) -> &Self
    where
        T: AsRef<str>,
    {
        self.config.holder = holder.as_ref().into();
        self
    }

    // TODO: Validate year
    pub fn year(&mut self, year: u16) -> &Self {
        self.config.year = year;
        self
    }

    // TODO: Validate email
    pub fn email(&mut self, email: Option<String>) -> &Self {
        self.config.email = email;
        self
    }

    pub fn format(&mut self, format: CopyrightNoticeFormat) -> &Self {
        self.config.format = format;
        self
    }

    pub fn exclude<T>(&mut self, patterns: Vec<T>) -> &Self
    where
        T: AsRef<str>,
    {
        let exclude = patterns.iter().map(|p| p.as_ref().to_string());
        self.config.exclude.extend(exclude);
        self
    }

    pub fn project(&mut self, project: Option<String>) -> &Self {
        self.config.project = project;
        self
    }

    pub fn project_url(&mut self, project_url: Option<url::Url>) -> &Self {
        self.config.project_url = project_url;
        self
    }

    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
}
