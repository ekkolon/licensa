use std::borrow::Cow;

use crate::terminal::Result;

pub trait Step {
    fn start(&mut self) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
    fn done(&self) -> bool;
    fn running(&self) -> bool;
    fn pending(&self) -> bool;
    fn message(&self) -> Cow<'_, str>;
}
