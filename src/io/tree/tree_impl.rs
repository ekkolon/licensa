use std::path::PathBuf;

use super::{DocumentSnapshot, DocumentState};

#[derive(Clone)]
pub struct TreeSnapshot {
    pub(super) src_root: PathBuf,
    pub(super) failed: Vec<DocumentSnapshot>,
    pub(super) unlicensed: Vec<DocumentSnapshot>,
    pub(super) modified: Vec<DocumentSnapshot>,
    pub(super) licensed: Vec<DocumentSnapshot>,
}

impl TreeSnapshot {
    pub fn count(&self) -> usize {
        self.count_failed() + self.count_licensed() + self.count_unlicensed()
    }

    pub fn count_licensed(&self) -> usize {
        self.licensed.len()
    }

    pub fn count_modified(&self) -> usize {
        self.modified.len()
    }

    pub fn count_failed(&self) -> usize {
        self.failed.len()
    }

    pub fn count_unlicensed(&self) -> usize {
        self.unlicensed.len()
    }

    pub fn get_state(&self, state: DocumentState) -> Vec<DocumentSnapshot> {
        let snapshots: &[DocumentSnapshot] = match state {
            DocumentState::Failed => self.failed.as_ref(),
            DocumentState::Licensed => self.licensed.as_ref(),
            DocumentState::Modified => self.modified.as_ref(),
            DocumentState::Unlicensed => self.unlicensed.as_ref(),
        };

        let mut snapshots: Vec<DocumentSnapshot> = snapshots
            .iter() // This borrows each snapshot mutably
            .map(|snapshot| {
                let mut snap = snapshot.clone();
                snap.strip_path_prefix(&self.src_root); // Modify the snapshot in place
                snap
            })
            .collect();

        self.sort_snapshots_by_path(snapshots.as_mut_slice());

        snapshots
    }

    fn sort_snapshots_by_path(&self, snapshots: &mut [DocumentSnapshot]) {
        snapshots.sort_by(|a, b| {
            a.path()
                .to_str()
                .unwrap_or("")
                .cmp(b.path().to_str().unwrap_or(""))
        })
    }
}
