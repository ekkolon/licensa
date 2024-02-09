// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::sync::Arc;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use ignore::{DirEntry, WalkParallel, WalkState};

/// A type representing a result of walking through a workspace.
pub type WalkResult = Result<DirEntry, ignore::Error>;

/// A function type representing a visitor for walking through a workspace.
pub type FnVisitor<'s> = Box<dyn FnMut(WalkResult) -> WalkState + Send + 's>;

type WalkPredicate = Arc<dyn Fn(WalkResult) -> bool + Send + Sync + 'static>;

/// Represents a workspace walker.
pub struct Walk {
    inner: WalkParallel,
    max_capacity: Option<usize>,
    quit_while: WalkPredicate,
    send_while: WalkPredicate,
}

impl Walk {
    pub fn new(inner: WalkParallel, max_capacity: Option<usize>) -> Self {
        Self {
            inner,
            max_capacity,
            quit_while: Arc::new(|_| false),
            send_while: Arc::new(|_| true),
        }
    }

    /// Runs the workspace walk with the provided visitor function.
    pub fn run<'a, F>(self, visit: F)
    where
        F: FnMut() -> FnVisitor<'a>,
    {
        self.inner.run(visit)
    }

    /// Runs the workspace walk and returns a receiver for collecting results.
    pub fn run_task(self) -> Receiver<WalkResult> {
        let (tx, rx) = self.chan::<WalkResult>();
        self.inner.run(|| {
            let tx = tx.clone();
            let quit_fn = self.quit_while.clone();
            let send_fn = self.send_while.clone();
            Box::new(move |result| {
                if quit_fn(result.clone()) {
                    return WalkState::Quit;
                }
                if send_fn(result.clone()) {
                    tx.send(result.clone()).unwrap();
                }
                WalkState::Continue
            })
        });

        rx
    }

    /// Sets a condition for sending results while walking the workspace.
    #[inline]
    pub fn send_while<T>(&mut self, when: T) -> &Self
    where
        T: Fn(WalkResult) -> bool + Sync + Send + 'static,
    {
        self.send_while = Arc::new(when);
        self
    }

    /// Sets a condition for quitting the workspace walk.
    #[inline]
    pub fn quit_while<T>(&mut self, when: T) -> &Self
    where
        T: Fn(WalkResult) -> bool + Sync + Send + 'static,
    {
        self.quit_while = Arc::new(when);
        self
    }

    /// Sets the maximum capacity of the channel for collecting results.
    #[inline]
    pub fn max_capacity(&mut self, limit: Option<usize>) -> &Self {
        if limit.is_none() && self.max_capacity.is_none() {
            return self;
        }
        self.max_capacity = limit;
        self
    }

    #[inline]
    fn chan<T>(&self) -> (Sender<T>, Receiver<T>) {
        match &self.max_capacity {
            None => crossbeam_channel::unbounded::<T>(),
            Some(cap) => crossbeam_channel::bounded::<T>(*cap),
        }
    }
}
