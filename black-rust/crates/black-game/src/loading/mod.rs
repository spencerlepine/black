//! Asset loading (G3): Bevy `AssetServer` over the extracted `assets/` tree (D4), typed loaders
//! for the RON tables / atlases / text banks (`types`), the `manifest.json` reader (`manifest`),
//! and the `Screen::Loading` state that waits for the required assets before entering `Boot`.
//!
//! Required set: `manifest.json`, every table in `assets/data/` (`black_data::loader::file`) and,
//! once the manifest is read, every `text/en/*.json` bank it lists. Sprites, audio and models are
//! loaded on demand by the screens that use them (their paths come from the manifest).
//!
//! A failed or missing asset never blocks the app: it is logged, listed in `LoadReport`, and the
//! app proceeds to `Boot` (like the UI font, which also degrades). That keeps the scripted
//! capture path (tools/ui-diff, partial `--only fonts` trees) working without a full extraction.
//!
//! Serving: native reads `$BLACK_ASSETS` (default `./assets`) from disk; wasm fetches the
//! relative URL `assets/<path>` (the page's directory). See crates/black-game/README.md.

// The handles and assets are consumed by later phases; the binary cannot see that yet.
#![allow(dead_code)]

pub mod manifest;
pub mod types;

use bevy::asset::{AssetMetaCheck, LoadState, UntypedHandle};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::flow::Screen;
use crate::{LAYER_BOTTOM, SCREEN_W};
use types::{AssetTypesPlugin, GameTables, ManifestAsset, TextBankAsset};

/// Bevy's `AssetPlugin` pointed at the extracted tree. Native: an absolute path
/// (`$BLACK_ASSETS` or `./assets`); wasm: the relative URL `assets`. Meta files are never
/// requested (the extractor writes none, and on the web each probe would be a 404).
pub fn asset_plugin() -> AssetPlugin {
    #[cfg(not(target_arch = "wasm32"))]
    let file_path = {
        let dir = crate::ui::assets_dir();
        std::env::current_dir()
            .map_or(dir.clone(), |cwd| cwd.join(dir))
            .to_string_lossy()
            .into_owned()
    };
    #[cfg(target_arch = "wasm32")]
    let file_path = "assets".to_string();
    AssetPlugin {
        file_path,
        meta_check: AssetMetaCheck::Never,
        ..default()
    }
}

/// Run condition: the app is past `Screen::Loading` (frame counters and scripted input start
/// when loading ends, so the capture timeline does not depend on how long loading took).
pub fn not_loading(screen: Res<State<Screen>>) -> bool {
    *screen.get() != Screen::Loading
}

/// Outcome of loading, readable by any later system.
#[derive(Resource, Default, Debug)]
pub struct LoadReport {
    /// Assets tracked so far (grows by the text banks once the manifest is read).
    pub total: usize,
    pub loaded: usize,
    /// Asset paths that failed to load.
    pub failed: Vec<String>,
    /// Required paths the manifest does not list.
    pub unlisted: Vec<String>,
}

impl LoadReport {
    /// Fraction in `0.0..=1.0` of tracked assets that finished (loaded or failed).
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.loaded + self.failed.len()) as f32 / self.total as f32
        }
    }
}

/// The manifest, once loaded (for finding sprite/audio/model paths on demand).
#[derive(Resource, Clone)]
pub struct GameManifest(pub Handle<ManifestAsset>);

/// Handles of the text banks listed by the manifest (`text/en/*.json`), in manifest order.
#[derive(Resource, Default, Clone)]
pub struct TextBanks(pub Vec<Handle<TextBankAsset>>);

#[derive(Resource)]
struct Tracker {
    manifest: Handle<ManifestAsset>,
    tracked: Vec<UntypedHandle>,
    planned: bool,
}

fn start_loading(mut commands: Commands, server: Res<AssetServer>) {
    let tables = GameTables::load(&server);
    let manifest: Handle<ManifestAsset> = server.load("manifest.json");
    let mut tracked = tables.untyped();
    tracked.push(manifest.clone().untyped());
    commands.insert_resource(tables);
    commands.insert_resource(GameManifest(manifest.clone()));
    commands.insert_resource(TextBanks::default());
    commands.insert_resource(LoadReport {
        total: tracked.len(),
        ..default()
    });
    commands.insert_resource(Tracker {
        manifest,
        tracked,
        planned: false,
    });
}

/// Update, `Screen::Loading` only: extend the required set from the manifest, count finished
/// assets, and leave for `Boot` when every tracked asset has loaded or failed.
fn poll_loading(
    server: Res<AssetServer>,
    manifests: Res<Assets<ManifestAsset>>,
    mut tracker: ResMut<Tracker>,
    mut banks: ResMut<TextBanks>,
    mut report: ResMut<LoadReport>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !tracker.planned {
        match server.load_state(&tracker.manifest) {
            LoadState::Loaded => {
                tracker.planned = true;
                if let Some(m) = manifests.get(&tracker.manifest) {
                    let m = &m.0;
                    report.unlisted = required_data_paths()
                        .into_iter()
                        .filter(|p| !m.contains(p))
                        .collect();
                    for path in m.paths_in("text/en/", ".json") {
                        let h: Handle<TextBankAsset> = server.load(path.to_string());
                        tracker.tracked.push(h.clone().untyped());
                        banks.0.push(h);
                    }
                }
            }
            LoadState::Failed(_) => tracker.planned = true,
            _ => {}
        }
    }
    report.total = tracker.tracked.len();
    report.loaded = 0;
    report.failed.clear();
    for h in &tracker.tracked {
        match server.load_state(h.id()) {
            LoadState::Loaded => report.loaded += 1,
            LoadState::Failed(_) => report.failed.push(
                server
                    .get_path(h.id())
                    .map_or_else(|| format!("{:?}", h.id()), |p| p.to_string()),
            ),
            _ => {}
        }
    }
    if tracker.planned && report.loaded + report.failed.len() == report.total {
        for p in &report.failed {
            warn!("asset failed to load: {p} (run `black extract-assets`, set BLACK_ASSETS)");
        }
        for p in &report.unlisted {
            warn!("asset not in manifest.json: {p}");
        }
        info!(
            "assets: {} loaded, {} failed, {} unlisted",
            report.loaded,
            report.failed.len(),
            report.unlisted.len()
        );
        next.set(Screen::Boot);
    }
}

/// `data/<table>.ron` for every table the game requires.
pub fn required_data_paths() -> Vec<String> {
    use black_data::loader::file as f;
    [
        f::PERSONAL,
        f::LEARNSET,
        f::EVOLUTION,
        f::MOVES,
        f::ITEMS,
        f::TRAINERS,
        f::ENCOUNTERS,
        f::GROWTH,
        f::TYPES,
        f::TMHM,
        f::EGG_MOVES,
        f::TUTORS,
    ]
    .iter()
    .map(|n| format!("data/{n}"))
    .collect()
}

/// Asset types, loaders, and the loading logic (no rendering; usable headless).
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AssetTypesPlugin)
            .init_resource::<LoadReport>()
            .add_systems(OnEnter(Screen::Loading), start_loading)
            .add_systems(Update, poll_loading.run_if(in_state(Screen::Loading)));
    }
}

/// Progress bar on the bottom screen while `Screen::Loading`.
pub struct ProgressScreenPlugin;

#[derive(Component)]
struct ProgressFill;

const BAR_W: f32 = 160.0;
const BAR_H: f32 = 8.0;

impl Plugin for ProgressScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Loading), spawn_bar)
            .add_systems(Update, update_bar.run_if(in_state(Screen::Loading)));
    }
}

fn spawn_bar(mut commands: Commands) {
    let scoped = DespawnOnExit(Screen::Loading);
    // z above the fade rects (100), which start fully black.
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3),
            Vec2::new(BAR_W + 4.0, BAR_H + 4.0),
        ),
        Transform::from_xyz(0.0, 0.0, 200.0),
        RenderLayers::layer(LAYER_BOTTOM),
        scoped.clone(),
    ));
    commands.spawn((
        Sprite::from_color(Color::BLACK, Vec2::new(BAR_W, BAR_H)),
        Transform::from_xyz(0.0, 0.0, 201.0),
        RenderLayers::layer(LAYER_BOTTOM),
        scoped.clone(),
    ));
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(0.0, BAR_H)),
        Transform::from_xyz(-BAR_W / 2.0, 0.0, 202.0),
        RenderLayers::layer(LAYER_BOTTOM),
        ProgressFill,
        scoped,
    ));
    debug_assert!(BAR_W < SCREEN_W as f32);
}

fn update_bar(
    report: Res<LoadReport>,
    mut fill: Single<(&mut Sprite, &mut Transform), With<ProgressFill>>,
) {
    let w = BAR_W * report.fraction();
    fill.0.custom_size = Some(Vec2::new(w, BAR_H));
    fill.1.translation.x = -BAR_W / 2.0 + w / 2.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    fn fixture_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("black-g3-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("data")).unwrap();
        std::fs::create_dir_all(d.join("text/en")).unwrap();
        d
    }

    fn headless(dir: &Path) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            AssetPlugin {
                file_path: dir.to_string_lossy().into_owned(),
                meta_check: AssetMetaCheck::Never,
                ..default()
            },
        ))
        .init_state::<Screen>()
        .add_plugins(AssetsPlugin);
        app
    }

    fn run_until_boot(app: &mut App) {
        let t = Instant::now();
        while *app.world().resource::<State<Screen>>().get() == Screen::Loading {
            app.update();
            assert!(
                t.elapsed() < Duration::from_secs(20),
                "loading did not finish"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        app.update();
    }

    #[test]
    fn loads_fixture_tree_and_reports_failures() {
        let dir = fixture_dir("tree");
        let growth =
            black_data::growth::GrowthTables(vec![black_data::growth::GrowthTable(vec![0, 1, 8])]);
        std::fs::write(
            dir.join("data/growth.ron"),
            black_data::to_ron(&growth).unwrap(),
        )
        .unwrap();
        std::fs::write(dir.join("data/types.ron"), "not ron").unwrap();
        std::fs::write(dir.join("text/en/sys_000.json"), "{\"entries\": []}").unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            "{\n  \"format\": 1,\n  \"converter_version\": \"t\",\n  \"files\": [\n    {\"path\": \"data/growth.ron\", \"category\": \"data\", \"size\": 1, \"sha1\": \"x\"},\n    {\"path\": \"text/en/sys_000.json\", \"category\": \"text\", \"size\": 1, \"sha1\": \"y\"}\n  ]\n}\n",
        )
        .unwrap();

        let mut app = headless(&dir);
        run_until_boot(&mut app);

        assert_eq!(*app.world().resource::<State<Screen>>().get(), Screen::Boot);
        let tables = app.world().resource::<GameTables>().clone();
        let loaded = app.world().resource::<Assets<types::GrowthAsset>>();
        assert_eq!(loaded.get(&tables.growth).unwrap().0, growth);
        let report = app.world().resource::<LoadReport>();
        // 12 tables + manifest + 1 text bank tracked; 10 missing + 1 malformed fail.
        assert_eq!(report.total, 14);
        assert_eq!(report.failed.len(), 11, "{:?}", report.failed);
        assert!(report.failed.iter().any(|p| p == "data/types.ron"));
        assert_eq!(report.loaded, 3);
        assert_eq!(report.unlisted.len(), 11);
        let banks = app.world().resource::<TextBanks>();
        assert_eq!(banks.0.len(), 1);
        let texts = app.world().resource::<Assets<TextBankAsset>>();
        assert_eq!(texts.get(&banks.0[0]).unwrap().0, "{\"entries\": []}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_manifest_still_leaves_loading() {
        let dir = fixture_dir("empty");
        let mut app = headless(&dir);
        run_until_boot(&mut app);
        let report = app.world().resource::<LoadReport>();
        assert_eq!(report.loaded, 0);
        assert_eq!(report.failed.len(), 13);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn fraction_counts_finished_assets() {
        let r = LoadReport {
            total: 4,
            loaded: 1,
            failed: vec!["a".into()],
            unlisted: vec![],
        };
        assert_eq!(r.fraction(), 0.5);
        assert_eq!(LoadReport::default().fraction(), 0.0);
    }
}
