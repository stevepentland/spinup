//! The run config is a struct used to define the settings used to
//! invoke [`run_app`](fn.run_app.html)

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub(crate) log_level: &'static str,
    pub(crate) run_package_installs: bool,
    pub(crate) run_file_downloads: bool,
    pub(crate) run_snap_installs: bool,
    pub(crate) config_file_path: String,
    pub(crate) print_parsed: bool,
}

impl RunConfig {
    pub fn new(
        config_file_path: String,
        log_level: &'static str,
        run_package_installs: bool,
        run_file_downloads: bool,
        run_snap_installs: bool,
        print_parsed: bool,
    ) -> Self {
        RunConfig {
            log_level,
            run_package_installs,
            run_file_downloads,
            run_snap_installs,
            config_file_path,
            print_parsed,
        }
    }
}
