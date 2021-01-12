mod validator;
mod minecraft;
mod artifact;
mod util;
mod download;
mod generator;

use clap::{App, Arg};
use crate::validator::correct_forge_version;
use crate::minecraft::version::Libraries;
use crate::minecraft::fabric::FabricLoaderManifest;
use crate::minecraft::forge::ForgeManifest;
use crate::minecraft::GameType;

fn main() {
    env_logger::init();
    let matches = App::new("NSLauncher Profile Generator")
        .version("1.0")
        .author("Team NS")
        .about("Generate profile for NSLauncher")
        .arg(
            Arg::new("version")
                .short('v')
                .required(true)
                .long("version")
                .takes_value(true)
                .about("Minecraft Version")
        )
        .arg(
            Arg::new("profileName")
                .short('n')
                .required(true)
                .long("name")
                .takes_value(true)
                .about("Profile name")
        )
        .arg(
            Arg::new("serverName")
                .short('a')
                .required(true)
                .long("address")
                .takes_value(true)
                .about("Server address")
        )
        .arg(
            Arg::new("serverPort")
                .short('p')
                .required(true)
                .long("port")
                .takes_value(true)
                .about("Server port")
        )
        .arg(
            Arg::new("forge")
                .about("Forge Version")
                .long("forge")
                .takes_value(true)
                .conflicts_with("fabric")
                .validator(validator::correct_forge_version)
        )
        .arg(
            Arg::new("fabric")
                .about("Fabric Loader Version")
                .long("fabric")
                .takes_value(true)
                .conflicts_with("forge")
                .validator(validator::correct_fabric_version)
        )
        .get_matches();
    let profile_name = matches.value_of("profileName").expect("Can't get profile name");
    let game_version = matches.value_of("version").expect("Can't get version");
    let game_libraries = matches.value_of_t::<Libraries>("version").unwrap_or_else(|e| e.exit());
    let address = matches.value_of("serverName").expect("Can't get server name");
    let port = matches.value_of_t::<u32>("serverPort").unwrap_or_else(|e| e.exit());
    let fabric = matches.value_of_t::<FabricLoaderManifest>("fabric");
    let forge = matches.value_of_t::<ForgeManifest>("forge");
    let game_type = if let Ok(manifest) = fabric {
        GameType::Fabric(manifest)
    } else if let Ok(manifest) = forge {
        GameType::Forge(manifest)
    } else {
        GameType::Vanilla
    };
    generator::generate_profile(profile_name, game_version, game_libraries, address, port, game_type);
}
