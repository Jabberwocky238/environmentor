trait PluginT {
    fn check_exists(&self) -> bool;
    fn try_install(&self) -> Result<(), String>;
    fn try_uninstall(&self) -> Result<(), String>;
}

struct AnacondaPlugin {
    install_dir: String,
}

impl AnacondaPlugin {
    fn new(install_dir: &str) -> AnacondaPlugin {
        AnacondaPlugin {
            install_dir: install_dir.to_string(),
        }
    }
}
