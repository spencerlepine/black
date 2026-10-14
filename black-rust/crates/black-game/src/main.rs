//! The game app: Bevy shell with the two stacked 256x192 DS screens (G1).
//!
//! Each screen is drawn by its own low-res camera into an offscreen image
//! (nearest-neighbour). A composite camera shows both images as sprites, one
//! above the other, at the largest integer scale that fits the window, with
//! black letterboxing.

mod flow;
mod input;
mod screens;
mod ui;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ImageRenderTarget, RenderTarget, ScalingMode};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Width of one DS screen in pixels.
const SCREEN_W: u32 = 256;
/// Height of one DS screen in pixels.
const SCREEN_H: u32 = 192;
/// Combined size of the two stacked screens.
const STACK_H: u32 = SCREEN_H * 2;

/// Render layer used by the top-screen scene.
const LAYER_TOP: usize = 1;
/// Render layer used by the bottom-screen scene.
const LAYER_BOTTOM: usize = 2;

/// Largest integer scale (at least 1) at which a 256x384 stack fits in a
/// window of `w` x `h` physical pixels.
fn integer_scale(w: u32, h: u32) -> u32 {
    (w / SCREEN_W).min(h / STACK_H).max(1)
}

/// Marks the composite (window) camera.
#[derive(Component)]
struct CompositeCamera;

fn main() {
    let mut app = App::new();
    // Capture mode (tools/ui-diff): the clock is lock-stepped to one 60 Hz tick per render frame
    // and screenshots are keyed by that fixed frame counter (screens::FrameCount), so a headless
    // capture is deterministic.
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("BLACK_SCREENSHOT").is_ok() {
        app.init_resource::<ScreenshotQueue>()
            .add_systems(First, lockstep_clock.after(bevy::time::TimeSystems))
            .add_systems(
                FixedUpdate,
                screenshot_and_exit.after(screens::count_frames),
            )
            .add_systems(Update, flush_screenshots);
    }
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pokemon Black (Rust)".into(),
                    resolution: (SCREEN_W * 2, STACK_H * 2).into(),
                    canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        input::InputPlugin,
        screens::ScreensPlugin,
        ui::UiPlugin,
        ui::demo::DemoPlugin,
    ))
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(Startup, setup)
    .add_systems(Update, fit_to_window)
    .run();
}

fn screen_image(images: &mut Assets<Image>) -> Handle<Image> {
    let mut image = Image::new_target_texture(
        SCREEN_W,
        SCREEN_H,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

fn spawn_screen_camera(commands: &mut Commands, target: Handle<Image>, layer: usize, order: isize) {
    commands.spawn((
        Camera2d,
        Camera {
            order,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::Image(ImageRenderTarget::from(target)),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: SCREEN_W as f32,
                height: SCREEN_H as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(layer),
    ));
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let top = screen_image(&mut images);
    let bottom = screen_image(&mut images);

    spawn_screen_camera(&mut commands, top.clone(), LAYER_TOP, -2);
    spawn_screen_camera(&mut commands, bottom.clone(), LAYER_BOTTOM, -1);

    // Composite camera: 1 world unit = 1 screen pixel, window-size scaling.
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(0),
        CompositeCamera,
    ));
    let half = SCREEN_H as f32 / 2.0;
    commands.spawn((Sprite::from_image(top), Transform::from_xyz(0.0, half, 0.0)));
    commands.spawn((
        Sprite::from_image(bottom),
        Transform::from_xyz(0.0, -half, 0.0),
    ));
}

/// Frames to capture from `BLACK_SCREENSHOT_FRAMES="160,175,190"` (several, written as
/// `<path>_<frame>.png`) or `BLACK_SCREENSHOT_FRAME=30` (one, written to `<path>` itself).
/// Returns `(frames, multi)`.
#[cfg(not(target_arch = "wasm32"))]
fn screenshot_frames() -> (Vec<u32>, bool) {
    if let Ok(list) = std::env::var("BLACK_SCREENSHOT_FRAMES") {
        let mut frames: Vec<u32> = list
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        frames.sort_unstable();
        frames.dedup();
        if !frames.is_empty() {
            return (frames, true);
        }
    }
    let at = std::env::var("BLACK_SCREENSHOT_FRAME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    (vec![at], false)
}

/// Path of the capture for `frame`: `<path>_<frame>.png` in multi-frame mode, else `<path>`.
#[cfg(not(target_arch = "wasm32"))]
fn screenshot_path(path: &str, frame: u32, multi: bool) -> String {
    if !multi {
        return path.to_string();
    }
    match path.strip_suffix(".png") {
        Some(stem) => format!("{stem}_{frame}.png"),
        None => format!("{path}_{frame}.png"),
    }
}

/// Capture mode: advance the game clock by exactly one fixed timestep per render frame, so fixed
/// frame N is always the state shown by render frame N. Without this the fixed loop catches up in
/// bursts after a stall (window creation, shader compile) and several ticks share one image.
#[cfg(not(target_arch = "wasm32"))]
fn lockstep_clock(
    mut current: ResMut<Time>,
    mut virt: ResMut<Time<Virtual>>,
    fixed: Res<Time<Fixed>>,
) {
    if !virt.is_paused() {
        virt.pause();
    }
    let step = fixed.timestep();
    virt.advance_by(step);
    *current = virt.as_generic();
}

/// Fixed frames whose capture is due but not yet requested from the renderer. Several fixed
/// ticks can run between two render frames; they all get the image rendered after the last of
/// them (one `Screenshot` per render frame, copied to each frame's path).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource, Default)]
struct ScreenshotQueue(Vec<u32>);

/// Native only, `FixedUpdate`: with `BLACK_SCREENSHOT=<path.png>` set, queue the listed fixed
/// frames (`screens::FrameCount`, the 60 Hz game frame) and exit 30 frames after the last one.
#[cfg(not(target_arch = "wasm32"))]
fn screenshot_and_exit(
    frame: Res<screens::FrameCount>,
    mut queue: ResMut<ScreenshotQueue>,
    mut exit: MessageWriter<AppExit>,
) {
    if std::env::var("BLACK_SCREENSHOT").is_err() {
        return;
    }
    let (frames, _) = screenshot_frames();
    let last = *frames.last().expect("at least one frame");
    if frames.contains(&frame.0) {
        queue.0.push(frame.0);
    } else if frame.0 == last + 30 {
        exit.write(AppExit::Success);
    }
}

/// Native only, `Update` (once per render frame): request one screenshot for the queued fixed
/// frames and write it to every queued frame's path.
#[cfg(not(target_arch = "wasm32"))]
fn flush_screenshots(mut commands: Commands, mut queue: ResMut<ScreenshotQueue>) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
    if queue.0.is_empty() {
        return;
    }
    let Ok(path) = std::env::var("BLACK_SCREENSHOT") else {
        return;
    };
    let (_, multi) = screenshot_frames();
    let frames = std::mem::take(&mut queue.0);
    let first = screenshot_path(&path, frames[0], multi);
    let copies: Vec<String> = frames[1..]
        .iter()
        .map(|&f| screenshot_path(&path, f, multi))
        .collect();
    let mut save = save_to_disk(first.clone());
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>| {
            save(captured);
            for p in &copies {
                if let Err(e) = std::fs::copy(&first, p) {
                    error!("screenshot copy {first} -> {p}: {e}");
                }
            }
        },
    );
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod screenshot_tests {
    use super::screenshot_path;

    #[test]
    fn multi_frame_paths_get_the_frame_suffix() {
        assert_eq!(
            screenshot_path("/tmp/x/shot.png", 160, true),
            "/tmp/x/shot_160.png"
        );
        assert_eq!(screenshot_path("/tmp/x/shot", 7, true), "/tmp/x/shot_7.png");
        assert_eq!(
            screenshot_path("/tmp/x/shot.png", 30, false),
            "/tmp/x/shot.png"
        );
    }
}

/// Keep the composite view at an integer scale, centred (letterboxed).
fn fit_to_window(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Single<&mut Projection, With<CompositeCamera>>,
) {
    let scale = integer_scale(window.physical_width(), window.physical_height());
    // WindowSize maps 1 unit to 1 logical pixel, so one unit shown as `scale`
    // physical pixels needs projection scale = scale_factor / scale.
    let proj_scale = window.scale_factor() / scale as f32;
    if let Projection::Orthographic(o) = &mut **cam {
        if (o.scale - proj_scale).abs() > f32::EPSILON {
            o.scale = proj_scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::integer_scale;

    #[test]
    fn scale_is_integer_and_fits() {
        assert_eq!(integer_scale(512, 768), 2);
        assert_eq!(integer_scale(1920, 1080), 2);
        assert_eq!(integer_scale(1000, 3000), 3);
        assert_eq!(integer_scale(100, 100), 1);
    }
}
