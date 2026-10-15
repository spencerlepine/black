//! Bevy asset types and loaders over the extracted `assets/` tree (D4).
//!
//! Each RON table is a newtype asset whose loader is just `black_data::loader::load_<table>`
//! (bytes in, table out, wasm-safe: the asset server hands over the bytes, no `std::fs`).
//! Parsing and validation live in `black-data`; nothing here knows what a table means (D6).

use std::fmt;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, Handle, LoadContext};
use bevy::prelude::{App, AssetServer, Plugin, TypePath};

use super::manifest::Manifest;

/// Why an asset failed to load (I/O or parse error, already rendered to text).
#[derive(Debug)]
pub struct AssetFailure(pub String);

impl fmt::Display for AssetFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AssetFailure {}

async fn read_bytes(reader: &mut dyn Reader) -> Result<Vec<u8>, AssetFailure> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| AssetFailure(e.to_string()))?;
    Ok(bytes)
}

/// Declares, per table: the asset newtype, its loader and a `GameTables` handle field; plus
/// `GameTables::load` (starts every table load).
macro_rules! tables {
    ($($field:ident: $asset:ident / $loader:ident = $ty:ty, $func:ident, $file:ident;)*) => {
        $(
            #[derive(Asset, TypePath)]
            pub struct $asset(pub $ty);

            #[derive(Default, TypePath)]
            pub struct $loader;

            impl AssetLoader for $loader {
                type Asset = $asset;
                type Settings = ();
                type Error = AssetFailure;

                async fn load(
                    &self,
                    reader: &mut dyn Reader,
                    _settings: &(),
                    _load_context: &mut LoadContext<'_>,
                ) -> Result<$asset, AssetFailure> {
                    let bytes = read_bytes(reader).await?;
                    black_data::loader::$func(&bytes)
                        .map($asset)
                        .map_err(|e| AssetFailure(e.to_string()))
                }

                fn extensions(&self) -> &[&str] {
                    &["ron"]
                }
            }
        )*

        /// Handles of every data table (`assets/data/*.ron`).
        #[derive(bevy::prelude::Resource, Clone)]
        pub struct GameTables {
            $(pub $field: Handle<$asset>,)*
        }

        impl GameTables {
            /// Starts loading every table through the asset server.
            pub fn load(server: &AssetServer) -> GameTables {
                GameTables {
                    $($field: server.load(format!("data/{}", black_data::loader::file::$file)),)*
                }
            }

            /// Untyped handles, for load tracking.
            pub fn untyped(&self) -> Vec<bevy::asset::UntypedHandle> {
                vec![$(self.$field.clone().untyped(),)*]
            }
        }

        fn register_tables(app: &mut App) {
            $(app.init_asset::<$asset>().init_asset_loader::<$loader>();)*
        }
    };
}

tables! {
    personal: PersonalAsset / PersonalLoader = black_data::personal::PersonalTable, load_personal, PERSONAL;
    learnset: LearnsetAsset / LearnsetLoader = black_data::learnset::LearnsetTable, load_learnset, LEARNSET;
    evolution: EvolutionAsset / EvolutionLoader = black_data::evolution::EvolutionTable, load_evolution, EVOLUTION;
    moves: MovesAsset / MovesLoader = black_data::moves::MoveTable, load_moves, MOVES;
    items: ItemsAsset / ItemsLoader = black_data::items::ItemTable, load_items, ITEMS;
    trainers: TrainersAsset / TrainersLoader = black_data::trainers::TrainerTable, load_trainers, TRAINERS;
    encounters: EncountersAsset / EncountersLoader = black_data::encounters::EncounterData, load_encounters, ENCOUNTERS;
    growth: GrowthAsset / GrowthLoader = black_data::growth::GrowthTables, load_growth, GROWTH;
    types: TypesAsset / TypesLoader = black_data::types::TypeChart, load_types, TYPES;
    tmhm: TmHmAsset / TmHmLoader = black_data::tmhm::TmHmTable, load_tmhm, TMHM;
    egg_moves: EggMovesAsset / EggMovesLoader = black_data::egg_moves::EggMoveTable, load_egg_moves, EGG_MOVES;
    tutors: TutorsAsset / TutorsLoader = black_data::tutors::TutorTable, load_tutors, TUTORS;
}

/// A text bank (`text/en/*.json`, written by `Bank::to_json`): validated UTF-8, kept as text.
#[derive(Asset, TypePath)]
pub struct TextBankAsset(pub String);

#[derive(Default, TypePath)]
pub struct TextBankLoader;

impl AssetLoader for TextBankLoader {
    type Asset = TextBankAsset;
    type Settings = ();
    type Error = AssetFailure;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<TextBankAsset, AssetFailure> {
        let bytes = read_bytes(reader).await?;
        String::from_utf8(bytes)
            .map(TextBankAsset)
            .map_err(|e| AssetFailure(e.to_string()))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// A sprite atlas description (`sprites/**/*.atlas.ron`). The RON schema belongs to the
/// consumer, so the bytes are kept; parse with `black_data::loader::load::<T>(&atlas.0)`.
#[derive(Asset, TypePath)]
pub struct AtlasAsset(pub Vec<u8>);

#[derive(Default, TypePath)]
pub struct AtlasLoader;

impl AssetLoader for AtlasLoader {
    type Asset = AtlasAsset;
    type Settings = ();
    type Error = AssetFailure;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<AtlasAsset, AssetFailure> {
        let bytes = read_bytes(reader).await?;
        std::str::from_utf8(&bytes).map_err(|e| AssetFailure(e.to_string()))?;
        Ok(AtlasAsset(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["atlas.ron"]
    }
}

/// `manifest.json`, parsed.
#[derive(Asset, TypePath)]
pub struct ManifestAsset(pub Manifest);

#[derive(Default, TypePath)]
pub struct ManifestLoader;

impl AssetLoader for ManifestLoader {
    type Asset = ManifestAsset;
    type Settings = ();
    type Error = AssetFailure;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ManifestAsset, AssetFailure> {
        let bytes = read_bytes(reader).await?;
        let text = std::str::from_utf8(&bytes).map_err(|e| AssetFailure(e.to_string()))?;
        Manifest::parse(text)
            .map(ManifestAsset)
            .map_err(AssetFailure)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// Registers every asset type and loader.
pub struct AssetTypesPlugin;

impl Plugin for AssetTypesPlugin {
    fn build(&self, app: &mut App) {
        register_tables(app);
        app.init_asset::<TextBankAsset>()
            .init_asset_loader::<TextBankLoader>()
            .init_asset::<AtlasAsset>()
            .init_asset_loader::<AtlasLoader>()
            .init_asset::<ManifestAsset>()
            .init_asset_loader::<ManifestLoader>();
    }
}
