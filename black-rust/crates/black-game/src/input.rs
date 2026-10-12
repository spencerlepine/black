//! Input mapping (G4): keyboard/gamepad -> DS buttons, mouse/touch -> bottom-screen touch.
//!
//! Default keyboard map: arrows = D-pad, Z = A, X = B, S = X, A = Y, Q = L, W = R,
//! Enter = Start, Right Shift / Backspace = Select. Gamepad (DS face layout, by
//! position): East = A, South = B, North = X, West = Y, shoulders = L/R,
//! Start/Select, D-pad. Touch: left mouse button or the first touch point, only
//! while inside the bottom screen.

// Public API consumed by later phases (G2+); the binary crate cannot see that yet.
#![allow(dead_code)]

use bevy::camera::visibility::RenderLayers;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{integer_scale, LAYER_BOTTOM, LAYER_TOP, SCREEN_H, SCREEN_W, STACK_H};

/// Seconds a button must be held before the first repeat.
pub const REPEAT_DELAY: f32 = 0.4;
/// Seconds between repeats after the first.
pub const REPEAT_RATE: f32 = 0.1;

/// One DS button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsButton {
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

impl DsButton {
    /// All buttons, in bit order.
    pub const ALL: [DsButton; 12] = [
        DsButton::A,
        DsButton::B,
        DsButton::X,
        DsButton::Y,
        DsButton::L,
        DsButton::R,
        DsButton::Start,
        DsButton::Select,
        DsButton::Up,
        DsButton::Down,
        DsButton::Left,
        DsButton::Right,
    ];

    fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// Number of repeat events in the half-open interval `(prev, now]` of hold time
/// (seconds), firing at `delay`, `delay + rate`, `delay + 2*rate`, ...
pub fn repeat_count(prev: f32, now: f32, delay: f32, rate: f32) -> u32 {
    let n = |t: f32| -> u32 {
        // Small epsilon absorbs f32 rounding at exact tick boundaries.
        if t + 1e-4 < delay || rate <= 0.0 {
            0
        } else {
            (((t - delay) / rate) + 1e-3).floor().max(0.0) as u32 + 1
        }
    };
    n(now).saturating_sub(n(prev))
}

/// DS button state, updated once per frame in `PreUpdate`.
#[derive(Resource, Default, Debug, Clone)]
pub struct DsButtons {
    held: u16,
    prev: u16,
    repeat: u16,
    hold_time: [f32; 12],
    /// Seconds before the first repeat.
    pub delay: f32,
    /// Seconds between repeats.
    pub rate: f32,
}

impl DsButtons {
    /// New state with the default repeat timing.
    pub fn new() -> Self {
        Self {
            delay: REPEAT_DELAY,
            rate: REPEAT_RATE,
            ..default()
        }
    }

    /// Advance one frame given the currently held mask and the frame time.
    pub fn update(&mut self, held: u16, dt: f32) {
        self.prev = self.held;
        self.held = held;
        self.repeat = 0;
        for b in DsButton::ALL {
            let i = b as usize;
            if held & b.bit() == 0 {
                self.hold_time[i] = 0.0;
            } else if self.prev & b.bit() == 0 {
                self.hold_time[i] = 0.0;
                self.repeat |= b.bit();
            } else {
                let prev = self.hold_time[i];
                self.hold_time[i] += dt;
                if repeat_count(prev, self.hold_time[i], self.delay, self.rate) > 0 {
                    self.repeat |= b.bit();
                }
            }
        }
    }

    /// Is the button currently down?
    pub fn held(&self, b: DsButton) -> bool {
        self.held & b.bit() != 0
    }
    /// Went down this frame.
    pub fn just_pressed(&self, b: DsButton) -> bool {
        self.held & !self.prev & b.bit() != 0
    }
    /// Went up this frame.
    pub fn just_released(&self, b: DsButton) -> bool {
        self.prev & !self.held & b.bit() != 0
    }
    /// True on the initial press and on each key-repeat tick while held.
    pub fn repeated(&self, b: DsButton) -> bool {
        self.repeat & b.bit() != 0
    }
}

/// Map a cursor position in physical window pixels to bottom-screen pixels
/// (0..256, 0..192), through the centred integer-scaled letterbox layout.
/// `None` when outside the bottom screen.
pub fn window_to_bottom(cursor: Vec2, win_w: u32, win_h: u32) -> Option<Vec2> {
    let s = integer_scale(win_w, win_h) as f32;
    let ox = (win_w as f32 - SCREEN_W as f32 * s) / 2.0;
    let oy = (win_h as f32 - STACK_H as f32 * s) / 2.0;
    let x = (cursor.x - ox) / s;
    let y = (cursor.y - oy) / s - SCREEN_H as f32;
    let inside = (0.0..SCREEN_W as f32).contains(&x) && (0.0..SCREEN_H as f32).contains(&y);
    inside.then_some(Vec2::new(x, y))
}

/// Bottom-screen touch state (pixel coords, origin top-left).
#[derive(Resource, Default, Debug, Clone)]
pub struct Touch {
    /// Position while held (None when not touching the screen).
    pub pos: Option<Vec2>,
    /// Last known position (kept for the release frame).
    pub last: Option<Vec2>,
    pressed: bool,
    released: bool,
}

impl Touch {
    /// Advance one frame with the current position (None = not touching).
    pub fn update(&mut self, now: Option<Vec2>) {
        self.pressed = self.pos.is_none() && now.is_some();
        self.released = self.pos.is_some() && now.is_none();
        if now.is_some() {
            self.last = now;
        }
        self.pos = now;
    }
    /// Touch began this frame.
    pub fn just_pressed(&self) -> bool {
        self.pressed
    }
    /// Touch ended this frame.
    pub fn just_released(&self) -> bool {
        self.released
    }
    /// Currently touching.
    pub fn held(&self) -> bool {
        self.pos.is_some()
    }
}

fn read_buttons(
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    time: Res<Time<Real>>,
    mut ds: ResMut<DsButtons>,
) {
    use DsButton as B;
    let key_map = [
        (KeyCode::KeyZ, B::A),
        (KeyCode::KeyX, B::B),
        (KeyCode::KeyS, B::X),
        (KeyCode::KeyA, B::Y),
        (KeyCode::KeyQ, B::L),
        (KeyCode::KeyW, B::R),
        (KeyCode::Enter, B::Start),
        (KeyCode::ShiftRight, B::Select),
        (KeyCode::Backspace, B::Select),
        (KeyCode::ArrowUp, B::Up),
        (KeyCode::ArrowDown, B::Down),
        (KeyCode::ArrowLeft, B::Left),
        (KeyCode::ArrowRight, B::Right),
    ];
    let pad_map = [
        (GamepadButton::East, B::A),
        (GamepadButton::South, B::B),
        (GamepadButton::North, B::X),
        (GamepadButton::West, B::Y),
        (GamepadButton::LeftTrigger, B::L),
        (GamepadButton::RightTrigger, B::R),
        (GamepadButton::Start, B::Start),
        (GamepadButton::Select, B::Select),
        (GamepadButton::DPadUp, B::Up),
        (GamepadButton::DPadDown, B::Down),
        (GamepadButton::DPadLeft, B::Left),
        (GamepadButton::DPadRight, B::Right),
    ];
    let mut held = 0u16;
    for (k, b) in key_map {
        if keys.pressed(k) {
            held |= b.bit();
        }
    }
    for pad in &pads {
        for (g, b) in pad_map {
            if pad.pressed(g) {
                held |= b.bit();
            }
        }
    }
    ds.update(held, time.delta_secs());
}

fn read_touch(
    window: Single<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut touch: ResMut<Touch>,
) {
    let sf = window.scale_factor();
    let raw = if let Some(t) = touches.iter().next() {
        Some(t.position())
    } else if mouse.pressed(MouseButton::Left) {
        window.cursor_position()
    } else {
        None
    };
    let now = raw
        .and_then(|p| window_to_bottom(p * sf, window.physical_width(), window.physical_height()));
    touch.update(now);
}

#[derive(Component)]
struct TouchDot;
#[derive(Component)]
struct ButtonText;

fn setup(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 1.0, 0.2), Vec2::splat(6.0)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Visibility::Hidden,
        RenderLayers::layer(LAYER_BOTTOM),
        TouchDot,
    ));
    commands.spawn((
        Text2d::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(10.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -85.0, 10.0),
        RenderLayers::layer(LAYER_TOP),
        ButtonText,
    ));
}

fn show(
    ds: Res<DsButtons>,
    touch: Res<Touch>,
    mut dot: Single<(&mut Transform, &mut Visibility), With<TouchDot>>,
    mut text: Single<&mut Text2d, With<ButtonText>>,
) {
    match touch.pos {
        Some(p) => {
            dot.0.translation.x = p.x - SCREEN_W as f32 / 2.0;
            dot.0.translation.y = SCREEN_H as f32 / 2.0 - p.y;
            *dot.1 = Visibility::Visible;
        }
        None => *dot.1 = Visibility::Hidden,
    }
    let held: Vec<String> = DsButton::ALL
        .iter()
        .filter(|b| ds.held(**b))
        .map(|b| format!("{b:?}"))
        .collect();
    let t = match touch.pos {
        Some(p) => format!("touch {:.0},{:.0}", p.x, p.y),
        None => "touch -".into(),
    };
    text.0 = format!("held: {}\n{t}", held.join(" "));
}

/// Registers `DsButtons`, `Touch` and the input test-scene display.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DsButtons::new())
            .init_resource::<Touch>()
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, (read_buttons, read_touch))
            .add_systems(Update, show);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_scale2_exact_window() {
        // 512x768 window, scale 2, no letterbox. Bottom screen starts at y=384.
        assert_eq!(
            window_to_bottom(Vec2::new(0.0, 384.0), 512, 768),
            Some(Vec2::new(0.0, 0.0))
        );
        assert_eq!(
            window_to_bottom(Vec2::new(256.0, 576.0), 512, 768),
            Some(Vec2::new(128.0, 96.0))
        );
        assert_eq!(window_to_bottom(Vec2::new(256.0, 383.9), 512, 768), None);
        assert_eq!(window_to_bottom(Vec2::new(512.0, 500.0), 512, 768), None);
        assert_eq!(window_to_bottom(Vec2::new(10.0, 768.0), 512, 768), None);
    }

    #[test]
    fn mapping_letterbox() {
        // 1000x900: scale 2 -> 512x768, ox=244, oy=66; bottom top edge y=66+384=450.
        assert_eq!(window_to_bottom(Vec2::new(243.9, 500.0), 1000, 900), None);
        assert_eq!(
            window_to_bottom(Vec2::new(244.0, 450.0), 1000, 900),
            Some(Vec2::ZERO)
        );
        assert_eq!(window_to_bottom(Vec2::new(500.0, 449.9), 1000, 900), None);
        assert_eq!(window_to_bottom(Vec2::new(500.0, 834.0), 1000, 900), None);
        assert!(window_to_bottom(Vec2::new(755.9, 833.9), 1000, 900).is_some());
    }

    #[test]
    fn mapping_tiny_window_clamps_scale_1() {
        // 100x100 window, scale 1; stack is larger than the window (negative offset).
        let p = window_to_bottom(Vec2::new(50.0, 50.0), 100, 100);
        // ox=-78, oy=-142 -> x=128, y=192-192=0
        assert_eq!(p, Some(Vec2::new(128.0, 0.0)));
    }

    #[test]
    fn repeat_timing() {
        assert_eq!(repeat_count(0.0, 0.39, 0.4, 0.1), 0);
        assert_eq!(repeat_count(0.39, 0.4, 0.4, 0.1), 1);
        assert_eq!(repeat_count(0.4, 0.49, 0.4, 0.1), 0);
        assert_eq!(repeat_count(0.49, 0.5, 0.4, 0.1), 1);
        assert_eq!(repeat_count(0.0, 0.7, 0.4, 0.1), 4);
    }

    #[test]
    fn buttons_edges_and_repeat() {
        let mut d = DsButtons::new();
        let a = DsButton::A.bit();
        d.update(a, 0.016);
        assert!(d.held(DsButton::A) && d.just_pressed(DsButton::A) && d.repeated(DsButton::A));
        d.update(a, 0.2);
        assert!(!d.just_pressed(DsButton::A) && !d.repeated(DsButton::A));
        d.update(a, 0.25);
        assert!(d.repeated(DsButton::A));
        d.update(0, 0.016);
        assert!(d.just_released(DsButton::A) && !d.held(DsButton::A));
    }

    #[test]
    fn touch_edges() {
        let mut t = Touch::default();
        t.update(Some(Vec2::new(1.0, 2.0)));
        assert!(t.just_pressed() && t.held());
        t.update(Some(Vec2::new(3.0, 2.0)));
        assert!(!t.just_pressed());
        t.update(None);
        assert!(t.just_released() && !t.held());
        assert_eq!(t.last, Some(Vec2::new(3.0, 2.0)));
    }
}
