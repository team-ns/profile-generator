use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{io, thread};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub fn download_file(url: &str, path: &str) {
    create_dir_all(path).unwrap();
    let url_parts: Vec<&str> = url.split('/').collect();
    let output = Path::new(path).join(url_parts.last().unwrap());
    match reqwest::blocking::get(url) {
        Ok(mut resp) => {
            match resp.status() {
                StatusCode::OK => (),
                _ => {
                    println!("Could not download this file: {}", url);
                    return;
                }
            }
            let mut file = match File::create(&output) {
                Ok(f) => f,
                Err(err) => {
                    println!(
                        "Error occurred while creating file: {} | Error: {}",
                        output.display(),
                        err
                    );
                    return;
                }
            };
            match io::copy(&mut resp, &mut file) {
                Ok(_) => {} //println!("File {} has been downloaded", output.display()),
                Err(err) => println!("Could not download this file: {} | Error: {}", url, err),
            }
        }

        Err(err) => println!("Could not download this file: {} | Error: {}", url, err),
    };
}

pub fn download_files_single(download: &Vec<(String, String)>) {
    for file in download {
        download_file(&file.0, &file.1)
    }
}


pub fn download_files_concurrent(download: &Vec<(String, String)>) {
    let workers: usize = 4;
    let chunks = download.chunks(workers);
    let mut threads = Vec::new();
    for chunk in chunks {
        let chunk: Vec<(String, String)> = chunk
            .iter()
            .map(|v| (v.0.to_string(), v.1.to_string()))
            .collect();
        threads.push(thread::spawn(move || {
            for file in chunk {
                download_file(&file.0, &file.1)
            }
        }));
    }
    for thread in threads {
        thread.join();
    }
}
