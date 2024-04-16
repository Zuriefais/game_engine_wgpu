use std::path;

use ecolor::{Color32, HexColor, Rgba};
use hashbrown::HashMap;
use log::{info, warn};
use ron::Deserializer;
use ron::*;
use serde::*;

use super::CellPhysicsType;
use std::fs;

#[derive(Clone, Deserialize, Debug)]
pub struct CellAsset {
    pub physics_behavior: CellPhysicsType,
    pub color: Rgba,
    pub name: String,
    pub density: i32,
}

#[derive(Default, Clone, Deserialize, Debug)]
pub struct CellAssets {
    pub assets: Vec<CellAsset>,
    pub assets_ids_map: HashMap<String, usize>,
    pub assets_color_vec: Vec<Rgba>,
    pub assets_physics_behavior_vec: Vec<CellPhysicsType>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigAsset {
    pub cell_paths: Vec<String>,
}

impl CellAssets {
    pub fn get_by_name(&self, name: String) -> Option<CellAsset> {
        if let Some(handle_index) = self.assets_ids_map.get(&name) {
            return self.assets.get(handle_index.clone()).cloned();
        }

        None
    }

    pub fn get_index_by_name(&self, name: String) -> Option<usize> {
        self.assets_ids_map.get(&name).copied()
    }

    pub fn get(&self, i: usize) -> Option<CellAsset> {
        self.assets.get(i).cloned()
    }

    pub fn get_color(&self, i: usize) -> Option<Rgba> {
        Some(self.assets_color_vec[i].clone())
    }

    pub fn get_physics_behavior(&self, i: usize) -> Option<CellPhysicsType> {
        Some(self.assets_physics_behavior_vec[i].clone())
    }

    pub fn get_color_by_name(&self, name: String) -> Option<Rgba> {
        if let Some(handle_index) = self.assets_ids_map.get(&name) {
            return Some(self.assets_color_vec[*handle_index].clone());
        }
        None
    }

    pub fn add(&mut self, asset: CellAsset) {
        self.assets.push(asset.clone());
        self.assets_color_vec.push(asset.color);
        self.assets_physics_behavior_vec
            .push(asset.physics_behavior);
        self.assets_ids_map
            .insert(asset.name, self.assets.len() - 1);
    }

    pub fn remove() {}

    pub fn get_last_index(self) -> usize {
        self.assets.len() - 1
    }
}

pub fn import_assets() -> Option<CellAssets> {
    let contents = fs::read_to_string("assets/config.config");

    if let Ok(paths_config_str) = contents {
        let config: ConfigAsset = toml::from_str(&paths_config_str).expect("Could't load config");

        let mut assets = CellAssets::default();
        info!("assets: {:?}", assets);

        for path in config.cell_paths {
            info!("trying to start load at path: {}", path);
            if let Some(asset) = import_asset(path) {
                assets.add(asset);
            }
        }

        return Some(assets);
    } else {
        info!("couldn't parse")
    }
    None
}

pub fn import_asset(path: String) -> Option<CellAsset> {
    let contents = fs::read_to_string("assets/".to_string() + &path);

    if let Ok(asset_str) = contents {
        info!("loading asset file");
        info!("{}", asset_str);

        let some_asset = toml::from_str::<CellAsset>(&asset_str);

        match some_asset {
            Ok(mut asset) => {
                asset.color = crate::utils::normalize_color(asset.color);
                info!("asset loaded: {:?}", asset);
                return Some(asset);
            }
            Err(error) => {
                warn!("{}", error.to_string())
            }
        }
    } else {
        info!("error loading file contents")
    }

    None
}
