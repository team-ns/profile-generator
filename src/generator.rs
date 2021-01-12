use crate::download::{download_file, download_files_concurrent, download_files_single};
use crate::minecraft::forge::LibraryType;
use crate::minecraft::version::Libraries;
use crate::minecraft::GameType;
use crate::minecraft::GameType::{Fabric, Forge};
use crate::util::{generate_download_url, generate_lib_path, get_yarn_path, get_yarn_url, jar_url};
use anyhow::Result;
use launcher_api::profile::Profile;
use std::collections::HashSet;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::iter::FromIterator;
use std::path::PathBuf;
use walkdir::WalkDir;
use zip::ZipArchive;

pub fn generate_profile(
    name: &str,
    version: &str,
    manifest: Libraries,
    address: &str,
    port: u32,
    game_type: GameType,
) -> Result<()> {
    let base = PathBuf::from(name);
    let native_folder = &base.join("natives").join(version);
    let assets_folder = &base.join("assets");
    let profile_folder = &base.join(name);
    let libraries_folder = base.join("libraries");
    let mut client_args = Vec::new();
    let mut main_class = "net/minecraft/client/main/Main".to_string();
    let mut classpath = Vec::new();
    std::fs::create_dir_all(&native_folder)?;
    std::fs::create_dir_all(&assets_folder)?;
    std::fs::create_dir_all(&profile_folder)?;
    std::fs::create_dir_all(&libraries_folder)?;
    log::info!("Download assets...");
    let assets = crate::util::get_assets(&manifest.asset_index.url).unwrap();
    let objects_path = assets_folder.join("objects");
    let mut assets_download = Vec::new();
    for (_, object) in assets.objects {
        let path = objects_path.join(&object.hash[0..2]);
        assets_download.push((
            format!(
                "http://resources.download.minecraft.net/{}/{}",
                &object.hash[0..2],
                object.hash
            ),
            path.to_str().unwrap().to_string(),
        ));
    }
    download_files_single(&assets_download);
    download_file(
        &manifest.asset_index.url,
        assets_folder.join("indexes").to_str().unwrap(),
    );
    log::info!("Download client...");
    download_file(
        &manifest.downloads.client.unwrap().url,
        &profile_folder.to_str().unwrap(),
    );
    std::fs::rename(
        profile_folder.join("client.jar").as_path(),
        profile_folder.join("minecraft.jar").as_path(),
    )?;
    classpath.push("minecraft.jar".to_string());
    log::info!("Download libs...");
    let mut profile_lib_paths = HashSet::new();
    let libs: Vec<(String, String)> = manifest
        .libraries
        .iter()
        .filter(|v| v.downloads.artifact.is_some())
        .map(|v| {
            let mut lib_path = PathBuf::from(
                v.downloads
                    .artifact
                    .as_ref()
                    .unwrap()
                    .path
                    .as_ref()
                    .unwrap()
                    .to_string(),
            );
            profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
            lib_path.pop();
            let url = v.downloads.artifact.as_ref().unwrap().url.to_string();
            let path = libraries_folder.join(lib_path);
            (url, path.to_str().unwrap().to_string())
        })
        .collect();
    download_files_concurrent(&libs);
    match game_type {
        Fabric(mut fabric_manifest) => {
            fabric_manifest
                .libraries
                .client
                .append(&mut fabric_manifest.libraries.common);
            let fabric_libs: Vec<(String, String)> = fabric_manifest
                .libraries
                .client
                .iter()
                .map(|v| {
                    let mut lib_path = PathBuf::from(generate_lib_path(&v.name));
                    profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                    lib_path.pop();
                    let url = generate_download_url(&v.url, &v.name);
                    let path = libraries_folder.join(lib_path);
                    (url, path.to_str().unwrap().to_string())
                })
                .collect();
            download_files_concurrent(&fabric_libs);
            let mappings_url = get_yarn_url(version);
            let mut lib_path = PathBuf::from(get_yarn_path(&version));
            profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
            lib_path.pop();
            let path = libraries_folder.join(lib_path);
            download_file(&mappings_url, path.to_str().unwrap());
            main_class = fabric_manifest.main_class.client;
        }
        Forge(forge_manifest) => {
            main_class = forge_manifest.main_class;
            let libs: Vec<(String, String)> = forge_manifest
                .libraries
                .iter()
                .filter_map(|v| match v {
                    LibraryType::PathLibrary(v) => Some(v),
                    _ => None,
                })
                .filter(|v| v.downloads.artifact.is_some())
                .filter_map(|v| {
                    // TODO: Add forge installer support
                    if v.downloads.artifact.as_ref().unwrap().path.is_some() {
                        let mut lib_path = PathBuf::from(
                            v.downloads
                                .artifact
                                .as_ref()
                                .unwrap()
                                .path
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        );
                        profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                        lib_path.pop();
                        let url = v.downloads.artifact.as_ref().unwrap().url.to_string();
                        let path = libraries_folder.join(lib_path);
                        return Some((url, path.to_str().unwrap().to_string()));
                    }
                    return None;
                })
                .collect();
            download_files_concurrent(&libs);
            let libs: Vec<(String, String)> = forge_manifest
                .libraries
                .iter()
                .filter_map(|v| match v {
                    LibraryType::NameLibrary(v) => Some(v),
                    _ => None,
                })
                .map(|v| {
                    let mut lib_path = PathBuf::from(generate_lib_path(&v.name));
                    profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                    lib_path.pop();
                    let url = generate_download_url(&v.url, &v.name);
                    let path = libraries_folder.join(lib_path);
                    (url, path.to_str().unwrap().to_string())
                })
                .collect();
            download_files_concurrent(&libs);
            // list of maven files to put in the libraries folder, but not in classpath
            if let Some(files) = forge_manifest.maven_files {
                let libs: Vec<(String, String)> = files
                    .iter()
                    .filter_map(|v| {
                        if v.downloads.artifact.as_ref().unwrap().path.is_some() {
                            let mut lib_path = PathBuf::from(
                                v.downloads
                                    .artifact
                                    .as_ref()
                                    .unwrap()
                                    .path
                                    .as_ref()
                                    .unwrap()
                                    .to_string(),
                            );
                            lib_path.pop();
                            let url = v.downloads.artifact.as_ref().unwrap().url.to_string();
                            let path = libraries_folder.join(lib_path);
                            return Some((url, path.to_str().unwrap().to_string()));
                        }
                        return None;
                    })
                    .collect();
                download_files_concurrent(&libs);
            }
            if let Some(tweakers) = forge_manifest.tweakers {
                for tweak in tweakers {
                    client_args.push("--tweakClass".to_string());
                    client_args.push(tweak);
                }
            }
        }
        _ => {}
    }
    log::info!("Download natives...");
    let temp_natives = base.join("natives_temp");
    create_dir_all(&temp_natives)?;
    let natives = manifest
        .libraries
        .iter()
        .filter(|v| v.downloads.classifiers.is_some())
        .flat_map(|v| {
            let mut natives: Vec<(String, String)> = Vec::new();
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_osx
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_windows
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_linux
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            natives
        })
        .collect::<Vec<(String, String)>>();
    download_files_concurrent(&natives);
    for entry in WalkDir::new(&temp_natives)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        if let Ok(mut zip) = ZipArchive::new(File::open(entry.path()).unwrap()) {
            for index in 0..zip.len() {
                let mut file = zip.by_index(index).unwrap();
                if file.is_file() {
                    if file.name().ends_with(".so")
                        || file.name().ends_with(".dll")
                        || file.name().ends_with(".dylib")
                    {
                        if let Ok(mut outfile) =
                            File::create(native_folder.join(&file.mangled_name()))
                        {
                            io::copy(&mut file, &mut outfile)?;
                        }
                    }
                }
            }
        }
    }
    remove_dir_all(temp_natives)?;
    log::info!("Generate json profile...");
    serde_json::to_writer_pretty(
        File::create(profile_folder.join(format!("profile.json"))).unwrap(),
        &Profile {
            name: name.to_string(),
            version: version.to_string(),
            libraries: Vec::from_iter(profile_lib_paths),
            class_path: classpath,
            main_class,
            update_verify: vec![],
            update_exclusion: vec![],
            jvm_args: vec![],
            client_args,
            assets: manifest.asset_index.id,
            assets_dir: "assets".to_string(),
            server_name: address.to_string(),
            server_port: port,
        },
    )?;
    Ok(())
}
