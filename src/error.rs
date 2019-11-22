#[derive(Debug, Clone)]
pub enum SpinupError {
    ConfigurationReadError(String),
    SystemDetailsError,
    //PackageInstallError(String, i32),
}
