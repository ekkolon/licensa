pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "invalid license period format. Expected a non-empty string in one of the following formats: YYYY, YYYY-YYYY, or YYYY-present. Found '{0}'"
    )]
    InvalidPeriodFormat(String),

    #[error("{0} does not represent a calendar year")]
    NotACalenderYear(String),

    #[error("the starting year {0} of a license period must be less than the ending year {1} of the period")]
    PeriodOutOfRange(u32, u32),

    #[error("SPDX license ID must be a non-empty string")]
    EmptyLicenseId,

    #[error("invalid SPDX License ID or expression: {0}")]
    InvslidLicenseIdOrExpression(String),

    #[error(transparent)]
    SpdxParseError(#[from] spdx::ParseError),

    #[error("No SPDX License found for '{0}'")]
    SpdxLicenseNotFound(String),
}
