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
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, screenshot_and_exit);
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

/// Native only: with `BLACK_SCREENSHOT=<path.png>` set, save a frame and exit.
#[cfg(not(target_arch = "wasm32"))]
fn screenshot_and_exit(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{save_to_disk, Screenshot};
    let Ok(path) = std::env::var("BLACK_SCREENSHOT") else {
        return;
    };
    *frame += 1;
    // BLACK_SCREENSHOT_FRAME (default 30) picks the frame to capture; exit 30 frames later.
    let at: u32 = std::env::var("BLACK_SCREENSHOT_FRAME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    if *frame == at {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    } else if *frame == at + 30 {
        exit.write(AppExit::Success);
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
