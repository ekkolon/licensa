pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Step is already running:\n{:>4}Step: {step}", "")]
    StepAlreadyRunning { step: String },

    #[error("Step is already finished:\n{:>4}Step: {step}", "")]
    StepAlreadyFinished { step: String },
}
