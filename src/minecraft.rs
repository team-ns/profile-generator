use crate::minecraft::forge::ForgeManifest;
use crate::minecraft::fabric::FabricLoaderManifest;

pub mod version;
pub mod fabric;
pub mod forge;
pub mod libraries;
pub mod assets;


pub enum GameType {
    Vanilla,
    Forge(ForgeManifest),
    Fabric(FabricLoaderManifest),
}