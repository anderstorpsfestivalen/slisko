use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_ENV: &str = "SLISKO_CONFIG";
const DEFAULT_CONFIG: &str = "configurations/9010.toml";

fn main() {
    println!("cargo:rerun-if-env-changed={CONFIG_ENV}");

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must set CARGO_MANIFEST_DIR"),
    );
    let repository_root = manifest_dir
        .parent()
        .expect("config must be a direct child of the repository root");
    let selected = env::var_os(CONFIG_ENV).unwrap_or_else(|| DEFAULT_CONFIG.into());
    let selected = PathBuf::from(selected);
    let config_path = if selected.is_absolute() {
        selected
    } else {
        repository_root.join(selected)
    };

    println!("cargo:rerun-if-changed={}", config_path.display());

    let generated = baker::bake_to_string(&config_path).unwrap_or_else(|error| {
        panic!(
            "failed to bake configuration {}: {error}",
            config_path.display()
        )
    });
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    write_if_changed(&out_dir.join("generated.rs"), generated.as_bytes());
}

fn write_if_changed(path: &Path, contents: &[u8]) {
    if fs::read(path).is_ok_and(|existing| existing == contents) {
        return;
    }
    fs::write(path, contents).unwrap_or_else(|error| {
        panic!(
            "failed to write generated config {}: {error}",
            path.display()
        )
    });
}
