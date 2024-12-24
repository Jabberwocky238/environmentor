trait PluginT {
    fn check_exists(&self) -> bool;
    fn try_install(&self) -> Result<(), String>;
}


struct AnacondaPlugin {
    name: String,
    version: String,
    executable: String,
    url: String,
    install_dir: String,
}

struct NVMPlugin {
    name: String,
    version: String,
    executable: String,
    url: String,
    install_dir: String,
}