// Copyright 2024 Nelson Dominguez
// SPDX-License-Identifier: Apache-2.0

//! This module provides a framework for processing files in parallel using Rust.
//!
//! # Overview
//!
//! - `IsContext`: A trait alias representing the context type that is `Send`, `'static`, and `Clone`.
//! - `Function`: A trait alias representing a function type that takes a mutable context and a string, returning a result.
//! - `FileTask`: A trait for processing file contents.
//! - `FileTaskClone`: A trait for cloning trait objects of `FileTask`.
//! - `FunctionFileTask`: A concrete implementation of `FileTask` using a function.
//! - `WorkTreeProcessor`: Orchestrates file processing in parallel using a collection of `FileTask` instances.
//!
//! # Example
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! // Define a custom context type
//! #[derive(Clone)]
//! struct MyContext;
//!
//! // Define a function for file processing
//! fn process_function(context: &mut MyContext, file_contents: &str) -> usize {
//!     // Your file processing logic here
//!     file_contents.len()
//! }
//!
//! // Create a WorkTreeProcessor and add a file processor
//! let mut processor = WorkTreeProcessor::new();
//! let context = MyContext;
//! let receiver = processor.add_file_processor(context.clone(), process_function);
//!
//! // Run the processor on a list of file paths
//! let file_paths = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
//! processor.run(file_paths);
//!
//! // Outputrieve results from the receiver
//! let result = receiver.recv().unwrap();
//! println!("Total processed result: {}", result);
//! ```

#![allow(dead_code)]
#![deny(bare_trait_objects)]

use crossbeam_channel::{Receiver, Sender};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    *,
};

/// Macro for defining trait aliases with optional type parameters and where clauses.
macro_rules! trait_aliases {(
    $(
            pub
            trait
            alias
            $Trait:ident
        $(
            (  $($ty_params:tt)*  )
        )?
            = {
                $($traits:tt)*
            }
        $(
            where {
                $($wc:tt)*
            }
        )?
            ;
    )*
) => (
    $(
        pub
        trait $Trait $(<$($ty_params)*>)? :
            $($traits)*
        $(
            where
                $($wc)*
        )?
        {}

        impl<Slf : ?Sized, $($($ty_params)*)?> $Trait $(<$($ty_params)*>)?
            for Slf
        where
            Slf : $($traits)*,
            $($($wc)*)?
        {}
    )*
)}

/// A trait representing a generic file processor.
///
/// Implementors of this trait should provide the logic for processing file contents.
///
/// # Examples
///
/// Implementing a custom file processor:
///
/// ```rust
/// # use crossbeam_channel::Sender;
/// # use std::sync::Arc;
/// # use rayon::prelude::*;
/// # use std::{fs, path::PathBuf};
/// # use std::sync::mpsc;
/// # use std::thread;
///
/// trait MyFileTask: FileTaskClone + Send {
///     fn process_file_contents(&mut self, file_contents: &str);
/// }
///
/// trait MyFileTaskClone {
///     fn clone_box(&self) -> Box<dyn MyFileTask>;
/// }
///
/// impl<T> MyFileTaskClone for T
/// where
///     T: MyFileTask + Clone + 'static,
/// {
///     fn clone_box(&self) -> Box<dyn MyFileTask> {
///         Box::new(self.clone())
///     }
/// }
/// ```
pub trait FileTask: FileTaskClone + Send {
    /// Processes the contents of a file.
    ///
    /// # Arguments
    ///
    /// * `file_contents` - A string slice representing the contents of the file.
    fn execute(&mut self, file_contents: &str);
}

/// A trait providing the ability to clone a `FileTask`.
pub trait FileTaskClone {
    /// Clones the `FileTask` instance.
    fn clone_box(&self) -> Box<dyn FileTask>;
}

/// Implementing `Clone` for `Box<dyn FileTask>`.
impl<T> FileTaskClone for T
where
    T: FileTask + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn FileTask> {
        Box::new(self.clone())
    }
}

/// Implementing `Clone` for `Box<dyn FileTask>`.
impl Clone for Box<dyn FileTask> {
    fn clone(&self) -> Box<dyn FileTask> {
        self.clone_box()
    }
}

// A trait alias representing a context that can be used in file processing.
trait_aliases! {
    pub trait alias Contextual = {
        Send + 'static +
        Clone
    };

    pub trait alias Function(Context, Output) = {
        Send + Sync + 'static +
        Fn(&mut Context, &str) -> Output
    } where {
        Context : Contextual,
        Output : Send + 'static,
    };
}

/// A struct representing a file processor based on a function.
pub struct FunctionFileTask<Context, Output>
where
    Context: Contextual,
    Output: Send + 'static,
{
    context: Context,
    function: Arc<dyn Function<Context, Output>>,
    results: Sender<Output>,
    completed: bool,
}

/// Implementing `Clone` for `FunctionFileTask`.
/// `Output` doesn't need to be `Clone`.
impl<Context, Output> Clone for FunctionFileTask<Context, Output>
where
    Context: Contextual,
    Output: Send + 'static,
{
    fn clone(&self) -> Self {
        let Self {
            context,
            function,
            results,
            completed,
        } = self;
        Self {
            context: context.clone(),
            function: function.clone(),
            results: results.clone(),
            completed: *completed,
        }
    }
}

/// Implementing `FileTask` for `FunctionFileTask`.
impl<Context, Output> FileTask for FunctionFileTask<Context, Output>
where
    Context: Contextual,
    Output: Send + 'static,
{
    fn execute(&mut self, file_contents: &str) {
        if self.completed {
            println!("Task completed");
            return;
        }

        let result = (*self.function)(&mut self.context, file_contents);
        let completed = self.results.send(result).is_err();

        self.completed = completed;
    }
}

/// Default implementations for `FunctionFileTask`.
impl<Context, Output> FunctionFileTask<Context, Output>
where
    Context: Contextual,
    Output: Send + 'static,
{
    pub fn new<F>(sender: Sender<Output>, context: Context, function: F) -> Self
    where
        F: Function<Context, Output>,
    {
        Self {
            context,
            function: Arc::new(function),
            results: sender,
            completed: false,
        }
    }
}

/// A struct representing a work tree processor.
///
/// This struct manages a collection of `FileTask` instances and provides a method
/// to run file processing on multiple paths concurrently.
pub struct FileTree {
    tasks: Vec<Box<dyn FileTask>>,
    paths: Vec<PathBuf>,
}

impl FileTree {
    /// Adds a file processor to the work tree processor.
    ///
    /// # Arguments
    ///
    /// * `context` - The context for file processing.
    /// * `function` - The function used to process file contents.
    ///
    /// # Outputurns
    ///
    /// A receiver for receiving results from the file processor.
    pub fn add_task<Context, Output, F>(
        &mut self,
        context: Context,
        function: F,
    ) -> Receiver<Output>
    where
        Context: Contextual,
        Output: Send + 'static,
        F: Function<Context, Output>,
    {
        let (sender, receiver) = crossbeam_channel::bounded(100);
        let task = FunctionFileTask::new(sender, context, function);
        self.tasks.push(Box::new(task));

        receiver
    }

    /// Adds a file processor to the work tree processor.
    ///
    /// # Arguments
    ///
    /// * `context` - The context for file processing.
    /// * `function` - The function used to process file contents.
    ///
    /// # Outputurns
    ///
    /// A receiver for receiving results from the file processor.
    pub fn add_task_for_path<P, Context, Output, F>(
        &mut self,
        path: P,
        context: Context,
        function: F,
    ) -> Receiver<Output>
    where
        P: AsRef<Path>,
        Context: Contextual,
        Output: Send + 'static,
        F: Function<Context, Output>,
    {
        self.paths.push(path.as_ref().into());
        self.add_task(context, function)
    }

    pub fn run(&self) {
        self.__private_run(self.paths.clone())
    }

    /// Runs file processing on the provided work tree paths.
    ///
    /// # Arguments
    ///
    /// * `tree_paths` - A vector of `PathBuf` representing the work tree paths.
    pub fn run_with_paths(&self, tree_paths: Vec<PathBuf>) {
        self.__private_run(tree_paths)
    }

    pub fn new() -> Self {
        Self {
            tasks: vec![],
            paths: vec![],
        }
    }

    /// Runs file processing on the provided work tree paths.
    ///
    /// # Arguments
    ///
    /// * `tree_paths` - A vector of `PathBuf` representing the work tree paths.
    fn __private_run(&self, tree_paths: Vec<PathBuf>) {
        let initial_tasks = self.tasks.clone();

        let read_file = |path: PathBuf| fs::read_to_string(path).ok();

        tree_paths
            .into_par_iter()
            .filter_map(read_file)
            .for_each_with(initial_tasks, |tasks, ref file_contents| {
                tasks
                    .iter_mut()
                    .for_each(|task| task.execute(file_contents))
            });
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use tempfile::TempDir;

    use super::*;

    // Mock context
    #[derive(Clone)]
    struct MockContext;

    // Mock function
    fn mock_function(_context: &mut MockContext, _file_contents: &str) -> i32 {
        // Mock processing logic
        42
    }

    fn create_temp_file<N: AsRef<Path>>(name: N) -> (TempDir, PathBuf) {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_file = &tmp_dir.path().to_path_buf();
        let tmp_file = tmp_file.join::<PathBuf>(name.as_ref().into());
        File::create(&tmp_file).unwrap();
        (tmp_dir, tmp_file.to_owned())
    }

    #[test]
    fn test_file_processor() {
        // Mock file processor implementation for testing
        #[derive(Clone)]
        struct MockFileTask;

        impl FileTask for MockFileTask {
            fn execute(&mut self, _file_contents: &str) {
                // Mock processing logic
            }
        }

        let processor = FileTree {
            tasks: vec![Box::new(MockFileTask)],
            paths: vec![],
        };

        // Run with an empty work tree path vector
        processor.run_with_paths(vec![]);
    }

    #[test]
    fn test_function_file_processor() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let function_processor = FunctionFileTask {
            context: MockContext,
            function: Arc::new(mock_function),
            results: sender,
            completed: false,
        };

        let mut cloned_processor = function_processor.clone();

        // Process file contents with the cloned processor
        cloned_processor.execute("example test content");
        assert!(!cloned_processor.completed);

        // Check if the sender sends a result
        assert_eq!(receiver.try_recv(), Ok(42));
    }

    #[test]
    fn test_work_tree_processor() {
        let mut work_tree_processor = FileTree {
            tasks: vec![],
            paths: vec![],
        };

        let receiver = work_tree_processor.add_task(MockContext, mock_function);

        let (tmp_dir, tmp_file) = create_temp_file("work_tree_processor.txt");

        // Run with an empty work tree path vector
        work_tree_processor.run_with_paths(vec![tmp_file]);

        // Check if the receiver receives the result
        assert_eq!(
            receiver.try_recv(),
            Ok(42),
            "Expected result of 42 from the receiver"
        );

        let _ = tmp_dir.close();
    }
}
