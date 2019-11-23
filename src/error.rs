// TODO: This is not good enough, spend some time on this soon

#[derive(Debug, Clone)]
pub enum SpinupError {
    ConfigurationReadError(String),
    SystemDetailsError,
    PackageInstallError(i32),
    ChildProcessSpawnError,
    NoPackageManagerForPlatform,
}
