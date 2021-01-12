use std::path::{Path, PathBuf};
use crate::artifact::Artifact;
use anyhow::Result;
use crate::minecraft::libraries::File as LibraryFile;
use crate::minecraft::assets::Assets;

pub fn jar_url(base_path: &Path, file: &LibraryFile) -> (String, String) {
    let url = file.url.clone();
    (url, base_path.to_str().unwrap().to_string())
}

pub fn get_assets(url: &str) -> Result<Assets> {
    let assets = reqwest::blocking::get(url)?.json::<Assets>()?;
    Ok(assets)
}

pub fn generate_download_url(base_url: &str, name: &str) -> String {
    format!("{}{}", base_url, &generate_lib_path(name))
}

pub fn generate_lib_path(name: &str) -> String {
    let artifact: Artifact = name.parse().unwrap();
    artifact.to_path().to_str().unwrap().to_string()
}

pub fn get_yarn_url(version: &str) -> String {
    format!(
        "https://maven.fabricmc.net/net/fabricmc/intermediary/{ver}/intermediary-{ver}.jar",
        ver = version
    )
}

pub fn get_yarn_path(version: &str) -> String {
    format!(
        "net/fabricmc/intermediary/{ver}/intermediary-{ver}.jar",
        ver = version)
}
