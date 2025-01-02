#[macro_export]
macro_rules! exit_with_error {
    ($kind:expr, $msg:expr) => {
        $crate::cli::Cli::command().error($kind, $msg).exit()
    };
}
