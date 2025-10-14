use std::{
    borrow::{Borrow, Cow},
    fs::{self},
    path::{Path, PathBuf},
};

use ignore::DirEntry;
use serde::Serialize;

use crate::{
    license::template::{
        copyright::SPDX_COPYRIGHT_NOTICE,
        header::extract_hash_bang,
        //header::{extract_hash_bang, HeaderDefinition, SourceHeaders},
    },
    Result,
};

use crate::io::template_cache::TemplateCache;

#[derive(Clone)]
pub struct DocumentRef {
    entry: DirEntry,
}

impl DocumentRef {
    pub fn new(entry: DirEntry) -> Self {
        DocumentRef { entry }
    }

    //pub fn path(&self) -> &Path {
    //    self.entry.path()
    //}

    pub fn read_to_snapshot(&self) -> Result<DocumentSnapshot> {
        let entry = self.read()?;
        Ok(entry.into_snapshot())
    }

    pub fn read(&self) -> Result<Document> {
        let path = self.entry.path();
        let content = fs::read_to_string(path)?;
        Ok(Document {
            content: Cow::Owned(content),
            path: self.entry.path().to_path_buf(),
            dry_run: false,
        })
    }

    pub fn read_dry_run(&self) -> Result<Document> {
        let path = self.entry.path();
        let content = fs::read_to_string(path)?;
        Ok(Document {
            content: Cow::Owned(content),
            path: self.entry.path().to_path_buf(),
            dry_run: true,
        })
    }

    /// Checks if a directory entry is a candidate for applying a license.
    pub fn is_licensable_file(&self) -> bool {
        let entry = &self.entry.borrow();

        // Only consider entry if it is a regular file
        if !entry.file_type().is_some_and(|ftype| ftype.is_file()) {
            return false;
        }

        let path = entry.path();
        if path.file_name().is_none() && path.extension().is_none() {
            return false;
        }

        true
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum DocumentSnapshot {
    Untouched { path: PathBuf },
    Modified { path: PathBuf },
    Licensed { path: PathBuf },
    Failed { path: PathBuf, reason: String },
}

impl DocumentSnapshot {
    pub fn licensed<P: AsRef<Path>>(path: P) -> Self {
        Self::Licensed {
            path: path.as_ref().into(),
        }
    }

    pub fn untouched<P: AsRef<Path>>(path: P) -> Self {
        Self::Untouched {
            path: path.as_ref().into(),
        }
    }

    pub fn modified<P: AsRef<Path>>(path: P) -> Self {
        Self::Modified {
            path: path.as_ref().into(),
        }
    }

    pub fn failed<P: AsRef<Path>, R: ToString>(path: P, reason: R) -> Self {
        Self::Failed {
            path: path.as_ref().into(),
            reason: reason.to_string(),
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            DocumentSnapshot::Modified { path, .. } => path.as_path(),
            DocumentSnapshot::Licensed { path, .. } => path.as_path(),
            DocumentSnapshot::Failed { path, .. } => path.as_path(),
            DocumentSnapshot::Untouched { path, .. } => path.as_path(),
        }
    }

    pub fn strip_path_prefix<P: AsRef<Path>>(&mut self, prefix: P) -> &Self {
        match self {
            DocumentSnapshot::Modified { ref mut path, .. }
            | DocumentSnapshot::Licensed { ref mut path, .. }
            | DocumentSnapshot::Untouched { ref mut path, .. }
            | DocumentSnapshot::Failed { ref mut path, .. } => {
                if let Ok(stripped_path) = path.strip_prefix(prefix) {
                    *path = stripped_path.into();
                }
            }
        }

        self
    }

    pub fn is_licensed(&self) -> bool {
        matches!(self, DocumentSnapshot::Licensed { .. })
    }

    pub fn is_untouched(&self) -> bool {
        matches!(self, DocumentSnapshot::Untouched { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, DocumentSnapshot::Failed { .. })
    }

    pub fn is_modified(&self) -> bool {
        matches!(self, DocumentSnapshot::Modified { .. })
    }
}

pub enum DocumentState {
    Failed,
    Unlicensed,
    Modified,
    Licensed,
}

pub struct Document {
    path: PathBuf,
    content: Cow<'static, str>,
    dry_run: bool,
}

const LICENSED_DOCUMENT_BREAKWORDS: &[&str] = &[
    "spdx-license-identifier: ",
    "copyright (c)",
    "all rights reserved",
    "mozilla public license",
    "academic free license",
    "gnu affero general public license",
    "gnu lesser general public license",
    "gnu free documentation license",
    "educational community license",
    "mulan psl v2",
    "copyright ",
];

impl Document {
    pub fn add_license<D: Serialize>(&self, data: D) -> Result<DocumentSnapshot> {
        if self.is_licensed() {
            return Ok(DocumentSnapshot::licensed(&self.path));
        }

        let template_key = self.get_path_suffix();
        let template = TemplateCache::compile(template_key, SPDX_COPYRIGHT_NOTICE, data)?;

        if self.dry_run {
            return Ok(DocumentSnapshot::modified(&self.path));
        }

        let content = self.add_license_header(&template.template);
        match fs::write(self.path(), content) {
            Err(err) => Ok(DocumentSnapshot::failed(&self.path, err.to_string())),
            Ok(_) => Ok(DocumentSnapshot::modified(&self.path)),
        }
    }

    pub fn into_snapshot(self) -> DocumentSnapshot {
        if self.is_licensed() {
            return DocumentSnapshot::licensed(&self.path);
        }

        DocumentSnapshot::untouched(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_licensed(&self) -> bool {
        let content = self.content.as_bytes();
        let n = std::cmp::min(1000, content.len());
        let lower_b: Vec<u8> = content[..n]
            .iter()
            .map(|&c| c.to_ascii_lowercase())
            .collect();

        let bytes = LICENSED_DOCUMENT_BREAKWORDS.iter().map(|w| w.as_bytes());

        for license in bytes {
            if lower_b
                .windows(license.len())
                .any(|window| window == license)
            {
                return true;
            }
        }

        false
    }

    fn add_license_header<H>(&self, header: H) -> Vec<u8>
    where
        H: AsRef<str>,
    {
        let template = header.as_ref().as_bytes().to_vec();
        let file_content = self.content.as_ref().as_bytes();
        let mut line = extract_hash_bang(file_content).unwrap_or_default();
        let mut content = file_content.to_vec();

        let line_break = b'\n';

        if !line.is_empty() {
            content = content.split_off(line.len());
            if line[line.len() - 1] != line_break {
                line.push(line_break);
            }
            content = [line, template, content].concat();
        } else {
            content = [template, content].concat();
        }

        content
    }

    // TODO: Implement src_root usage
    //fn get_template_def(&self) -> Option<&HeaderDefinition<'_>> {
    //    let lookup_name = self.get_path_suffix();
    //    SourceHeaders::find_header_definition_by_extension(&lookup_name)
    //}

    fn get_path_suffix(&self) -> String {
        self.path.extension().map_or_else(
            || {
                self.path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map_or(String::new(), |s| s.to_owned())
            },
            |extension| {
                let mut lookup_name = String::with_capacity(extension.len() + 1);
                lookup_name.push('.');
                lookup_name.push_str(extension.to_str().unwrap_or_default());
                lookup_name
            },
        )
    }
}
