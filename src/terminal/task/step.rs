use std::borrow::Cow;

use crate::terminal::Result;

pub trait Step {
    fn done(&self) -> bool;
    fn running(&self) -> bool;
    fn finish(&mut self) -> Result<()>;
    fn message(&self) -> Cow<'_, str>;
}

pub trait LazyStep: Step {
    fn start(&mut self) -> Result<()>;
}
