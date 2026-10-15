//! Placeholder screens, debug driver and fade rendering for the app state machine (G2).
//!
//! The pure logic (tables, fade curve) lives in `flow`; this module wires it to Bevy
//! `States`/`SubStates` and draws one text label + coloured rect per state on the two
//! DS screens.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::flow::{
    on_auto, on_button, Fade, FadeEvent, FlowConfig, Overlay, Request, Screen, FADE_LEVELS,
};
use crate::input::{DsButton, DsButtons, Touch};
use crate::{LAYER_BOTTOM, LAYER_TOP, SCREEN_H, SCREEN_W};

/// Frames spent in the current screen with no fade running (drives auto-advance).
#[derive(Resource, Default)]
struct Dwell(u32);

/// Scripted button presses `(fixed-frame, button)` for headless screenshots
/// (`BLACK_SCRIPT="70:A,180:Start"`). Frames count 60 Hz `FixedUpdate` ticks
/// (`FrameCount`), the same clock `BLACK_SCREENSHOT_FRAMES` uses.
#[derive(Resource, Default, Debug)]
pub struct Script(pub Vec<(u32, DsButton)>);

/// Number of 60 Hz fixed ticks since start (1 on the first tick). Shared by the
/// scripted input and the screenshot capture so both see the same frame index.
#[derive(Resource, Default, Debug)]
pub struct FrameCount(pub u32);

/// First system of `FixedUpdate`: advance the frame counter.
pub fn count_frames(mut frame: ResMut<FrameCount>) {
    frame.0 += 1;
}

/// Parse `"frame:Button,frame:Button"`. Unknown entries are skipped.
pub fn parse_script(s: &str) -> Vec<(u32, DsButton)> {
    s.split(',')
        .filter_map(|item| {
            let (f, b) = item.trim().split_once(':')?;
            let b = DsButton::ALL
                .into_iter()
                .find(|x| format!("{x:?}").eq_ignore_ascii_case(b.trim()))?;
            Some((f.trim().parse().ok()?, b))
        })
        .collect()
}

/// Full-screen black rectangle used for the brightness fade (one per DS screen).
#[derive(Component)]
struct FadeRect;

/// Panel belonging to the currently open overlay.
#[derive(Component)]
struct OverlayPanel;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        let mut cfg = FlowConfig::default();
        if let Some(n) = std::env::var("BLACK_FADE_FRAMES")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            cfg.fade_out_frames = n;
            cfg.fade_in_frames = n;
        }
        let script = std::env::var("BLACK_SCRIPT")
            .map(|s| parse_script(&s))
            .unwrap_or_default();
        app.init_state::<Screen>()
            .add_sub_state::<Overlay>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(cfg)
            // Boot starts black and fades in.
            .insert_resource(Fade::In {
                frame: 0,
                total: cfg.fade_in_frames,
            })
            .insert_resource(Script(script))
            .init_resource::<Dwell>()
            .init_resource::<FrameCount>()
            .add_systems(Startup, spawn_fade_rects)
            .add_systems(
                FixedUpdate,
                (count_frames, drive, tick_flow)
                    .chain()
                    .run_if(crate::loading::not_loading),
            )
            .add_systems(Update, (sync_overlay, apply_fade));
        for s in [
            Screen::Boot,
            Screen::Title,
            Screen::NewGame,
            Screen::Continue,
            Screen::Overworld,
            Screen::Battle,
        ] {
            app.add_systems(OnEnter(s), move |c: Commands| spawn_screen(c, s));
        }
    }
}

fn rect(layer: usize, color: Color, size: Vec2, xy: Vec2, z: f32) -> impl Bundle {
    (
        Sprite::from_color(color, size),
        Transform::from_xyz(xy.x, xy.y, z),
        RenderLayers::layer(layer),
    )
}

fn label(layer: usize, text: impl Into<String>, px: f32, xy: Vec2, z: f32) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont {
            font_size: bevy::text::FontSize::Px(px),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(xy.x, xy.y, z),
        RenderLayers::layer(layer),
    )
}

fn screen_size() -> Vec2 {
    Vec2::new(SCREEN_W as f32, SCREEN_H as f32)
}

fn spawn_fade_rects(mut commands: Commands) {
    for layer in [LAYER_TOP, LAYER_BOTTOM] {
        commands.spawn((
            rect(layer, Color::BLACK, screen_size(), Vec2::ZERO, 100.0),
            FadeRect,
        ));
    }
}

/// (top colour, bottom colour, title, hint) for a screen.
fn screen_style(s: Screen) -> (Color, Color, &'static str, &'static str) {
    match s {
        // Never spawned (no OnEnter hook): the progress bar is `loading::ProgressScreenPlugin`.
        Screen::Loading => (Color::BLACK, Color::BLACK, "", ""),
        Screen::Boot => (
            Color::srgb(0.1, 0.1, 0.1),
            Color::srgb(0.05, 0.05, 0.05),
            "BOOT",
            "system init...",
        ),
        Screen::Title => (
            Color::srgb(0.1, 0.2, 0.5),
            Color::srgb(0.05, 0.1, 0.25),
            "TITLE",
            "A: new game   Y: continue",
        ),
        Screen::NewGame => (
            Color::srgb(0.1, 0.45, 0.2),
            Color::srgb(0.05, 0.25, 0.1),
            "NEW GAME",
            "intro placeholder...",
        ),
        Screen::Continue => (
            Color::srgb(0.1, 0.45, 0.45),
            Color::srgb(0.05, 0.25, 0.25),
            "CONTINUE",
            "load placeholder...",
        ),
        Screen::Overworld => (
            Color::srgb(0.35, 0.5, 0.2),
            Color::srgb(0.2, 0.28, 0.1),
            "OVERWORLD",
            "Start: menu   X: battle",
        ),
        Screen::Battle => (
            Color::srgb(0.6, 0.15, 0.15),
            Color::srgb(0.3, 0.08, 0.08),
            "BATTLE",
            "A: back to overworld",
        ),
    }
}

fn spawn_screen(mut commands: Commands, s: Screen) {
    let (top, bottom, title, hint) = screen_style(s);
    let scoped = DespawnOnExit(s);
    commands.spawn((
        rect(LAYER_TOP, top, screen_size(), Vec2::ZERO, 0.0),
        scoped.clone(),
    ));
    commands.spawn((
        rect(
            LAYER_TOP,
            Color::srgb(0.9, 0.9, 0.9),
            Vec2::new(96.0, 48.0),
            Vec2::new(0.0, 20.0),
            1.0,
        ),
        scoped.clone(),
    ));
    commands.spawn((
        label(LAYER_TOP, title, 20.0, Vec2::new(0.0, -30.0), 2.0),
        scoped.clone(),
    ));
    commands.spawn((
        rect(LAYER_BOTTOM, bottom, screen_size(), Vec2::ZERO, 0.0),
        scoped.clone(),
    ));
    commands.spawn((
        label(LAYER_BOTTOM, hint, 12.0, Vec2::new(0.0, -70.0), 2.0),
        scoped,
    ));
}

/// Rebuild the overlay panels whenever the overlay (or its existence) changes.
fn sync_overlay(
    mut commands: Commands,
    overlay: Option<Res<State<Overlay>>>,
    panels: Query<Entity, With<OverlayPanel>>,
    mut shown: Local<Overlay>,
) {
    let now = overlay.map_or(Overlay::Closed, |o| *o.get());
    if now == *shown {
        return;
    }
    *shown = now;
    for e in &panels {
        commands.entity(e).despawn();
    }
    let (text, hint) = match now {
        Overlay::Closed => return,
        // The start menu is the real list-menu widget (ui::demo).
        Overlay::StartMenu => return,
        Overlay::Party => ("PARTY", "B: back"),
        Overlay::Bag => ("BAG", "B: back"),
        Overlay::Summary => ("SUMMARY", "B: back"),
        Overlay::PCBox => ("PC BOX", "B: back"),
        Overlay::Pokedex => ("POKEDEX", "B: back"),
        Overlay::Save => ("SAVE", "B: back"),
    };
    let scoped = DespawnOnExit(Screen::Overworld);
    // Bottom screen: menu panel. Sub-menus also take over the top screen.
    commands.spawn((
        rect(
            LAYER_BOTTOM,
            Color::srgb(0.15, 0.15, 0.35),
            Vec2::new(SCREEN_W as f32 - 16.0, SCREEN_H as f32 - 16.0),
            Vec2::ZERO,
            10.0,
        ),
        OverlayPanel,
        scoped.clone(),
    ));
    commands.spawn((
        label(LAYER_BOTTOM, text, 16.0, Vec2::new(0.0, 50.0), 11.0),
        OverlayPanel,
        scoped.clone(),
    ));
    commands.spawn((
        label(LAYER_BOTTOM, hint, 10.0, Vec2::new(0.0, -20.0), 11.0),
        OverlayPanel,
        scoped.clone(),
    ));
    if now != Overlay::StartMenu {
        commands.spawn((
            rect(
                LAYER_TOP,
                Color::srgb(0.25, 0.25, 0.55),
                screen_size(),
                Vec2::ZERO,
                10.0,
            ),
            OverlayPanel,
            scoped.clone(),
        ));
        commands.spawn((
            label(LAYER_TOP, text, 20.0, Vec2::ZERO, 11.0),
            OverlayPanel,
            scoped,
        ));
    }
}

/// Input -> transition requests, once per 60 Hz fixed tick. Input is ignored while a fade
/// runs. Scripted presses fire on the fixed frame they name (`FrameCount`).
#[allow(clippy::too_many_arguments)]
fn drive(
    ds: Res<DsButtons>,
    touch: Res<Touch>,
    script: Res<Script>,
    cfg: Res<FlowConfig>,
    screen: Res<State<Screen>>,
    overlay: Option<Res<State<Overlay>>>,
    mut fade: ResMut<Fade>,
    mut next_overlay: Option<ResMut<NextState<Overlay>>>,
    frame: Res<FrameCount>,
) {
    if !fade.is_idle() {
        return;
    }
    let cur_overlay = overlay.map_or(Overlay::Closed, |o| *o.get());
    let scripted = |b: DsButton| script.0.iter().any(|&(f, sb)| f == frame.0 && sb == b);
    for b in DsButton::ALL {
        // A tap on the bottom screen acts as A (debug).
        let pressed =
            ds.just_pressed(b) || scripted(b) || (b == DsButton::A && touch.just_pressed());
        if !pressed {
            continue;
        }
        match on_button(*screen.get(), cur_overlay, b) {
            Some(Request::Screen(s)) => {
                *fade = Fade::start(Request::Screen(s), &cfg);
                return;
            }
            Some(Request::Overlay(o)) => {
                if let Some(n) = next_overlay.as_mut() {
                    n.set(o);
                }
                return;
            }
            None => {}
        }
    }
}

/// 60 Hz tick: advance the fade, apply screen changes at full black, auto-advance placeholders.
fn tick_flow(
    cfg: Res<FlowConfig>,
    screen: Res<State<Screen>>,
    mut fade: ResMut<Fade>,
    mut dwell: ResMut<Dwell>,
    mut next: ResMut<NextState<Screen>>,
) {
    if let FadeEvent::Apply(Request::Screen(s)) = fade.tick(&cfg) {
        next.set(s);
        dwell.0 = 0;
        return;
    }
    if fade.is_idle() {
        dwell.0 += 1;
        if let Some(to) = on_auto(*screen.get()) {
            if dwell.0 >= cfg.dwell(*screen.get()) {
                *fade = Fade::start(Request::Screen(to), &cfg);
            }
        }
    }
}

fn apply_fade(fade: Res<Fade>, mut rects: Query<&mut Sprite, With<FadeRect>>) {
    let a = fade.darkness() as f32 / FADE_LEVELS as f32;
    for mut s in &mut rects {
        s.color = Color::srgba(0.0, 0.0, 0.0, a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_parsing() {
        assert_eq!(
            parse_script("70:A, 180:start,bogus,5:Zed"),
            vec![(70, DsButton::A), (180, DsButton::Start)]
        );
        assert!(parse_script("").is_empty());
    }
}
