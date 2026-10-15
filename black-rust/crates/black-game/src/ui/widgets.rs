//! Bevy half of the UI kit: font loading, glyph rendering, frames and the interactive widgets.
//!
//! Coordinates: every widget root sits at a DS-screen pixel `(x, y)` (origin top-left, y down) on
//! `layer` (`LAYER_TOP` / `LAYER_BOTTOM`); `px()` converts to the centred camera space. Glyph
//! sprites are 1:1 atlas rects at integer positions, so text is pixel-exact at native resolution
//! (the screen cameras are 256x192 with nearest-neighbour scaling).

use std::path::{Path, PathBuf};

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use black_text::layout::GlyphWidth;

use super::font::{FixedWidth, FontMetrics};
use super::list::{ListEvent, ListState, NavInput, NumberEvent, NumberInput};
use super::msgbox::{cells_to_units, MessageBox, MsgEvent, MsgOpts};
use crate::input::{DsButton, DsButtons};
use crate::screens::Script;
use crate::{SCREEN_H, SCREEN_W};

/// Font used for dialogue and menus (`font0` = 12x15; which font is used where is UNVERIFIED, A4b).
const FONT_INDEX: u32 = 0;
/// Alpha of the glyph shadow pixels (level 2 of the 2 bpp atlas).
const SHADOW_ALPHA: u8 = 90;
/// Pixels between two text lines (font line feed 15 + 1).
const LINE_PITCH: i32 = 16;

/// DS pixel (top-left origin) -> camera space (centre origin, y up).
pub fn px(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x - SCREEN_W as f32 / 2.0, SCREEN_H as f32 / 2.0 - y, z)
}

/// Loaded font: metrics + a white/alpha copy of the atlas (tinted per text with `Sprite::color`).
#[derive(Resource)]
pub struct UiFont {
    pub metrics: Option<FontMetrics>,
    pub atlas: Handle<Image>,
}

impl UiFont {
    /// Glyph advances for layout (fixed 6 px when no font is loaded).
    pub fn glyphs(&self) -> &dyn GlyphWidth {
        match &self.metrics {
            Some(m) => m,
            None => &FixedWidth(6),
        }
    }
}

/// User settings the widgets read.
#[derive(Resource, Debug, Clone)]
pub struct UiSettings {
    /// Text speed option 0..2 (`black_text::speed::option_speed`); `BLACK_TEXT_SPEED` overrides.
    pub text_speed: u32,
}

impl Default for UiSettings {
    fn default() -> Self {
        UiSettings {
            text_speed: std::env::var("BLACK_TEXT_SPEED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        }
    }
}

/// Widget input of this frame (cursor keys include DS key repeat).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct UiInput {
    pub nav: NavInput,
    /// Y just pressed (the demo's trigger).
    pub y: bool,
}

/// Ordering of the kit's systems; consumers run `.after(UiSet::Widgets)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSet {
    Input,
    Widgets,
    Render,
}

/// Text drawn with the bitmap font. Set `units` (game character codes, `0xFFFE` = new line) and
/// the glyph sprites are rebuilt.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct BitmapText {
    pub units: Vec<u16>,
    pub color: Color,
    pub spacing: i32,
    pub line_pitch: i32,
}

impl BitmapText {
    pub fn new(units: Vec<u16>) -> BitmapText {
        BitmapText {
            units,
            color: Color::srgb(0.1, 0.1, 0.12),
            spacing: 0,
            line_pitch: LINE_PITCH,
        }
    }
}

#[derive(Component)]
struct GlyphSprite;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiSettings>()
            .init_resource::<UiInput>()
            .configure_sets(
                Update,
                (UiSet::Input, UiSet::Widgets, UiSet::Render).chain(),
            )
            .add_systems(PreStartup, load_font)
            .add_systems(
                Update,
                collect_input
                    .in_set(UiSet::Input)
                    .run_if(crate::loading::not_loading),
            )
            .add_systems(
                Update,
                (update_msgbox, update_list, update_number).in_set(UiSet::Widgets),
            )
            .add_systems(FixedUpdate, tick_msgbox)
            .add_systems(Update, rebuild_text.in_set(UiSet::Render));
    }
}

/// Where the extracted assets live.
pub fn assets_dir() -> PathBuf {
    std::env::var_os("BLACK_ASSETS").map_or_else(|| PathBuf::from("assets"), PathBuf::from)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_font(
    dir: &Path,
    images: &mut Assets<Image>,
) -> Result<(FontMetrics, Handle<Image>), String> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

    let fonts = dir.join("fonts");
    let text = std::fs::read_to_string(fonts.join(format!("font{FONT_INDEX}.metrics.ron")))
        .map_err(|e| format!("{}: {e}", fonts.display()))?;
    let metrics = FontMetrics::parse(&text)?;
    let bytes = std::fs::read(fonts.join(&metrics.atlas)).map_err(|e| e.to_string())?;
    let mut img = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::RENDER_WORLD,
    )
    .map_err(|e| e.to_string())?;
    // The extractor writes black ink with `level * 255 / 3` in alpha (2 bpp). Level 1 is the
    // glyph body and level 2 its shadow (UNVERIFIED reading of the glyph dumps): make the ink
    // white so a sprite tint recolours it, body opaque, shadow translucent.
    if let Some(data) = img.data.as_mut() {
        for p in data.as_chunks_mut::<4>().0 {
            p[..3].fill(255);
            if metrics.bpp == 2 {
                p[3] = match (u32::from(p[3]) * 3 + 127) / 255 {
                    0 => 0,
                    2 => SHADOW_ALPHA,
                    _ => 255,
                };
            }
        }
    }
    Ok((metrics, images.add(img)))
}

#[cfg(target_arch = "wasm32")]
fn read_font(_: &Path, _: &mut Assets<Image>) -> Result<(FontMetrics, Handle<Image>), String> {
    // TODO(G3): fetch fonts over HTTP through the asset server.
    Err("font loading from assets/ is native-only until G3".into())
}

fn load_font(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let dir = assets_dir();
    match read_font(&dir, &mut images) {
        Ok((m, h)) => commands.insert_resource(UiFont {
            metrics: Some(m),
            atlas: h,
        }),
        Err(e) => {
            warn!("UI font not loaded ({e}); text will not be drawn. Run `black extract-assets --only fonts` and set BLACK_ASSETS.");
            commands.insert_resource(UiFont {
                metrics: None,
                atlas: Handle::default(),
            });
        }
    }
}

fn collect_input(
    ds: Res<DsButtons>,
    script: Option<Res<Script>>,
    mut input: ResMut<UiInput>,
    mut frame: Local<u32>,
) {
    *frame += 1;
    let scripted = |b: DsButton| {
        script
            .as_ref()
            .is_some_and(|s| s.0.iter().any(|&(f, sb)| f == *frame && sb == b))
    };
    let edge = |b: DsButton| ds.just_pressed(b) || scripted(b);
    let rep = |b: DsButton| ds.repeated(b) || scripted(b);
    input.nav = NavInput {
        up: rep(DsButton::Up),
        down: rep(DsButton::Down),
        left: rep(DsButton::Left),
        right: rep(DsButton::Right),
        a: edge(DsButton::A),
        b: edge(DsButton::B),
    };
    input.y = edge(DsButton::Y);
}

fn rebuild_text(
    mut commands: Commands,
    font: Res<UiFont>,
    texts: Query<(Entity, &BitmapText, Option<&Children>), Changed<BitmapText>>,
    glyphs: Query<(), With<GlyphSprite>>,
    layers: Query<&RenderLayers>,
) {
    let Some(m) = &font.metrics else { return };
    for (e, t, children) in &texts {
        for c in children.into_iter().flatten() {
            if glyphs.contains(*c) {
                commands.entity(*c).despawn();
            }
        }
        let layer = layers.get(e).cloned().unwrap_or_default();
        for p in m.place(&t.units, t.spacing, t.line_pitch) {
            let Some(g) = m.glyph(p.code) else { continue };
            let (ox, oy) = m.cell_origin(g.index);
            let child = commands
                .spawn((
                    Sprite {
                        image: font.atlas.clone(),
                        color: t.color,
                        rect: Some(Rect::new(
                            ox as f32,
                            oy as f32,
                            (ox + m.cell_w) as f32,
                            (oy + m.cell_h) as f32,
                        )),
                        ..default()
                    },
                    Anchor::TOP_LEFT,
                    Transform::from_xyz(p.x as f32, -(p.y as f32), 0.0),
                    layer.clone(),
                    GlyphSprite,
                ))
                .id();
            commands.entity(e).add_child(child);
        }
    }
}

// ---------------------------------------------------------------- frames and small shapes

/// One filled rectangle of a frame: offset from the frame's top-left, size, colour, z step.
pub type FrameRect = (Vec2, Vec2, Color, f32);

/// Placeholder window frame (no extracted frame graphics yet, see README "Window frame"):
/// dark outline, light-blue rim, white fill.
pub fn frame_rects(w: f32, h: f32) -> [FrameRect; 3] {
    [
        (
            Vec2::ZERO,
            Vec2::new(w, h),
            Color::srgb(0.16, 0.19, 0.28),
            0.0,
        ),
        (
            Vec2::splat(1.0),
            Vec2::new(w - 2.0, h - 2.0),
            Color::srgb(0.53, 0.63, 0.78),
            0.01,
        ),
        (
            Vec2::splat(3.0),
            Vec2::new(w - 6.0, h - 6.0),
            Color::WHITE,
            0.02,
        ),
    ]
}

fn pixel_rect(layer: &RenderLayers, off: Vec2, size: Vec2, color: Color, z: f32) -> impl Bundle {
    (
        Sprite::from_color(color, size),
        Anchor::TOP_LEFT,
        Transform::from_xyz(off.x, -off.y, z),
        layer.clone(),
    )
}

/// Rows `(x offset, width)` of a small triangle: pointing right (cursor) or down (advance arrow).
pub fn triangle_rows(right: bool, size: u32) -> Vec<(u32, u32)> {
    // `size` rows tall for a down arrow (width shrinks); `2 * size - 1` rows for a right arrow.
    if right {
        (0..2 * size - 1)
            .map(|i| (0, size - i.abs_diff(size - 1)))
            .collect()
    } else {
        (0..size).map(|i| (i, 2 * (size - i) - 1)).collect()
    }
}

fn spawn_triangle(
    commands: &mut Commands,
    layer: &RenderLayers,
    parent: Entity,
    right: bool,
    size: u32,
    color: Color,
) -> Entity {
    let root = commands
        .spawn((Transform::default(), Visibility::default(), layer.clone()))
        .id();
    for (i, (x, w)) in triangle_rows(right, size).into_iter().enumerate() {
        let r = commands
            .spawn(pixel_rect(
                layer,
                Vec2::new(x as f32, i as f32),
                Vec2::new(w as f32, 1.0),
                color,
                0.0,
            ))
            .id();
        commands.entity(root).add_child(r);
    }
    commands.entity(parent).add_child(root);
    root
}

fn spawn_frame(commands: &mut Commands, layer: &RenderLayers, parent: Entity, w: f32, h: f32) {
    for (off, size, color, z) in frame_rects(w, h) {
        let r = commands.spawn(pixel_rect(layer, off, size, color, z)).id();
        commands.entity(parent).add_child(r);
    }
}

fn spawn_text(
    commands: &mut Commands,
    layer: &RenderLayers,
    parent: Entity,
    at: Vec2,
    z: f32,
    units: Vec<u16>,
) -> Entity {
    let e = commands
        .spawn((
            BitmapText::new(units),
            Transform::from_xyz(at.x, -at.y, z),
            Visibility::default(),
            layer.clone(),
        ))
        .id();
    commands.entity(parent).add_child(e);
    e
}

fn root(commands: &mut Commands, layer: &RenderLayers, x: f32, y: f32, z: f32) -> Entity {
    commands
        .spawn((px(x, y, z), Visibility::default(), layer.clone()))
        .id()
}

fn text_width(font: &UiFont, units: &[u16]) -> i32 {
    black_text::layout::max_line_width(units, 0, font.glyphs()) as i32
}

// ---------------------------------------------------------------- message box

/// A dialogue box: frame + 2 lines of typewriter text + advance arrow.
#[derive(Component)]
pub struct MsgBoxWidget {
    pub logic: MessageBox,
    /// Takes A/B when true (otherwise the owner drives it).
    pub active: bool,
    /// Set when the last page was acknowledged; the owner despawns the box.
    pub finished: bool,
    text: Entity,
    arrow: Entity,
}

/// Standard dialogue box geometry: 240x48 at the bottom of a screen, 224 px of text.
pub const MSG_W: f32 = 240.0;
pub const MSG_H: f32 = 48.0;

/// Spawn a message box at `(x, y)` (top-left) with the standard size.
pub fn spawn_msgbox(
    commands: &mut Commands,
    font: &UiFont,
    settings: &UiSettings,
    layer: usize,
    (x, y): (f32, f32),
    text: &[u16],
) -> Entity {
    let layer = RenderLayers::layer(layer);
    let opts = MsgOpts {
        speed: black_text::speed::option_speed(settings.text_speed),
        ..MsgOpts::default()
    };
    let logic = MessageBox::new(text, &opts, font.glyphs());
    let e = root(commands, &layer, x, y, 20.0);
    spawn_frame(commands, &layer, e, MSG_W, MSG_H);
    let t = spawn_text(commands, &layer, e, Vec2::new(8.0, 7.0), 0.1, Vec::new());
    let arrow = spawn_triangle(commands, &layer, e, false, 4, Color::srgb(0.9, 0.2, 0.2));
    commands.entity(arrow).insert((
        Transform::from_xyz(MSG_W - 14.0, -(MSG_H - 12.0), 0.1),
        Visibility::Hidden,
    ));
    commands.entity(e).insert(MsgBoxWidget {
        logic,
        active: true,
        finished: false,
        text: t,
        arrow,
    });
    e
}

fn tick_msgbox(mut boxes: Query<&mut MsgBoxWidget>) {
    for mut b in &mut boxes {
        b.logic.tick();
    }
}

fn update_msgbox(
    input: Res<UiInput>,
    time: Res<Time<Real>>,
    mut boxes: Query<&mut MsgBoxWidget>,
    mut texts: Query<&mut BitmapText>,
    mut vis: Query<&mut Visibility>,
) {
    let blink = ((time.elapsed_secs() * 3.0) as u32).is_multiple_of(2);
    for mut b in &mut boxes {
        if b.active && (input.nav.a || input.nav.b) && b.logic.press() == MsgEvent::Finished {
            b.finished = true;
        }
        let units = cells_to_units(b.logic.visible());
        if let Ok(mut t) = texts.get_mut(b.text) {
            if t.units != units {
                t.units = units;
            }
        }
        if let Ok(mut v) = vis.get_mut(b.arrow) {
            let want = if b.logic.arrow() && blink {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *v != want {
                *v = want;
            }
        }
    }
}

// ---------------------------------------------------------------- list menu (also yes/no)

/// A vertical list menu with cursor. `event` is the result of the last frame's input.
#[derive(Component)]
pub struct ListWidget {
    pub state: ListState,
    pub labels: Vec<Vec<u16>>,
    pub active: bool,
    pub event: ListEvent,
    rows: Vec<Entity>,
    cursor: Entity,
    up: Entity,
    down: Entity,
}

const ROW_H: f32 = 16.0;

/// Spawn a list at `(x, y)`; `visible` rows are shown, the rest scroll.
pub fn spawn_list(
    commands: &mut Commands,
    font: &UiFont,
    layer: usize,
    (x, y): (f32, f32),
    labels: Vec<Vec<u16>>,
    visible: usize,
    wrap: bool,
) -> Entity {
    let layer = RenderLayers::layer(layer);
    let state = ListState::new(labels.len(), visible, wrap);
    let widest = labels
        .iter()
        .map(|l| text_width(font, l))
        .max()
        .unwrap_or(0);
    let w = widest as f32 + 28.0;
    let h = state.visible() as f32 * ROW_H + 12.0;
    let e = root(commands, &layer, x, y, 20.0);
    spawn_frame(commands, &layer, e, w, h);
    let red = Color::srgb(0.9, 0.2, 0.2);
    let rows = (0..state.visible())
        .map(|r| {
            spawn_text(
                commands,
                &layer,
                e,
                Vec2::new(16.0, 6.0 + r as f32 * ROW_H),
                0.1,
                Vec::new(),
            )
        })
        .collect();
    let cursor = spawn_triangle(commands, &layer, e, true, 4, Color::srgb(0.1, 0.1, 0.12));
    let up = spawn_triangle(commands, &layer, e, false, 3, red);
    let down = spawn_triangle(commands, &layer, e, false, 3, red);
    commands
        .entity(up)
        .insert(Transform::from_xyz(w - 10.0, -5.0, 0.1).with_scale(Vec3::new(1.0, -1.0, 1.0)));
    commands
        .entity(down)
        .insert(Transform::from_xyz(w - 10.0, -(h - 5.0), 0.1));
    commands.entity(e).insert(ListWidget {
        state,
        labels,
        active: true,
        event: ListEvent::None,
        rows,
        cursor,
        up,
        down,
    });
    e
}

fn update_list(
    input: Res<UiInput>,
    mut lists: Query<&mut ListWidget>,
    mut texts: Query<&mut BitmapText>,
    mut tf: Query<&mut Transform>,
    mut vis: Query<&mut Visibility>,
) {
    for mut l in &mut lists {
        l.event = if l.active {
            l.state.handle(&input.nav)
        } else {
            ListEvent::None
        };
        for (r, row) in l.rows.iter().enumerate() {
            let units = l
                .labels
                .get(l.state.scroll() + r)
                .cloned()
                .unwrap_or_default();
            if let Ok(mut t) = texts.get_mut(*row) {
                if t.units != units {
                    t.units = units;
                }
            }
        }
        if let Ok(mut t) = tf.get_mut(l.cursor) {
            t.translation = Vec3::new(5.0, -(6.0 + l.state.cursor_row() as f32 * ROW_H + 4.0), 0.1);
        }
        for (e, on) in [(l.up, l.state.more_above()), (l.down, l.state.more_below())] {
            if let Ok(mut v) = vis.get_mut(e) {
                *v = if on {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

// ---------------------------------------------------------------- number input

/// A digit editor box. `event` is the last frame's result.
#[derive(Component)]
pub struct NumberWidget {
    pub input: NumberInput,
    pub active: bool,
    pub event: NumberEvent,
    text: Entity,
    marker: Entity,
}

const DIGIT_W: f32 = 8.0;

/// Spawn a number box at `(x, y)`.
pub fn spawn_number(
    commands: &mut Commands,
    layer: usize,
    (x, y): (f32, f32),
    input: NumberInput,
) -> Entity {
    let layer = RenderLayers::layer(layer);
    let w = input.digits() as f32 * DIGIT_W + 20.0;
    let e = root(commands, &layer, x, y, 20.0);
    spawn_frame(commands, &layer, e, w, 28.0);
    let text = spawn_text(commands, &layer, e, Vec2::new(10.0, 6.0), 0.1, Vec::new());
    let marker = commands
        .spawn(pixel_rect(
            &layer,
            Vec2::ZERO,
            Vec2::new(DIGIT_W - 1.0, 2.0),
            Color::srgb(0.9, 0.2, 0.2),
            0.1,
        ))
        .id();
    commands.entity(e).add_child(marker);
    commands.entity(e).insert(NumberWidget {
        input,
        active: true,
        event: NumberEvent::None,
        text,
        marker,
    });
    e
}

fn update_number(
    input: Res<UiInput>,
    mut boxes: Query<&mut NumberWidget>,
    mut texts: Query<&mut BitmapText>,
    mut tf: Query<&mut Transform>,
) {
    for mut n in &mut boxes {
        n.event = if n.active {
            n.input.handle(&input.nav)
        } else {
            NumberEvent::None
        };
        // Digits are 6 px advance + 2 px spacing = the 8 px cell of the cursor marker.
        let units: Vec<u16> = n
            .input
            .digit_chars()
            .into_iter()
            .map(|d| u16::from(b'0') + u16::from(d))
            .collect();
        if let Ok(mut t) = texts.get_mut(n.text) {
            if t.units != units {
                t.units = units;
            }
            if t.spacing != 2 {
                t.spacing = 2;
            }
        }
        if let Ok(mut t) = tf.get_mut(n.marker) {
            t.translation = Vec3::new(10.0 + n.input.pos() as f32 * DIGIT_W, -22.0, 0.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_maps_corners() {
        let t = px(0.0, 0.0, 1.0).translation;
        assert_eq!((t.x, t.y, t.z), (-128.0, 96.0, 1.0));
        let t = px(256.0, 192.0, 0.0).translation;
        assert_eq!((t.x, t.y), (128.0, -96.0));
    }

    #[test]
    fn frame_layers_nest() {
        let r = frame_rects(40.0, 20.0);
        assert_eq!(r[0].1, Vec2::new(40.0, 20.0));
        assert_eq!(r[2].0, Vec2::splat(3.0));
        assert_eq!(r[2].1, Vec2::new(34.0, 14.0));
        assert!(r[0].3 < r[1].3 && r[1].3 < r[2].3);
    }

    #[test]
    fn triangles() {
        assert_eq!(triangle_rows(false, 3), vec![(0, 5), (1, 3), (2, 1)]);
        assert_eq!(
            triangle_rows(true, 3),
            vec![(0, 1), (0, 2), (0, 3), (0, 2), (0, 1)]
        );
    }
}
