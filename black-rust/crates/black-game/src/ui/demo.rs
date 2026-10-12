//! Demo wiring of the UI kit into the G2 placeholder flow (replaced when the real screens land):
//!
//! * Title: a message box with typewriter text on the bottom screen.
//! * Overworld start menu: a scrolling list menu (A opens a sub-overlay, B closes via `flow`).
//! * Overworld, Y: message box (3 pages) -> yes/no -> number input -> result message.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use super::font::encode;
use super::list::{yes_no_answer, NumberEvent, NumberInput};
use super::widgets::*;
use crate::flow::{Overlay, Screen};
use crate::{LAYER_BOTTOM, LAYER_TOP};

/// Start menu entries and the overlay each opens.
const START_ITEMS: [(&str, Overlay); 7] = [
    ("POKEDEX", Overlay::Pokedex),
    ("POKEMON", Overlay::Party),
    ("BAG", Overlay::Bag),
    ("PC BOX", Overlay::PCBox),
    ("SUMMARY", Overlay::Summary),
    ("SAVE", Overlay::Save),
    ("EXIT", Overlay::Closed),
];

/// Where the demo dialogue is.
#[derive(Resource, Default)]
enum Stage {
    #[default]
    Idle,
    Intro(Entity),
    Ask {
        msg: Entity,
        list: Option<Entity>,
    },
    Count {
        msg: Entity,
        num: Entity,
    },
    Outro(Entity),
}

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stage>()
            .add_systems(OnEnter(Screen::Title), spawn_title_dialog)
            .add_systems(OnEnter(Overlay::StartMenu), spawn_start_menu)
            .add_systems(
                Update,
                (start_menu_events, dialog_demo).after(UiSet::Widgets),
            )
            .add_systems(OnExit(Screen::Overworld), reset_stage);
    }
}

fn reset_stage(mut stage: ResMut<Stage>) {
    *stage = Stage::Idle;
}

fn spawn_title_dialog(mut commands: Commands, font: Res<UiFont>, settings: Res<UiSettings>) {
    let e = spawn_msgbox(
        &mut commands,
        &font,
        &settings,
        LAYER_BOTTOM,
        (8.0, 24.0),
        &encode("Welcome to the world of\nPokemon! Press A to begin."),
    );
    // Decorative: the title screen keeps its own A handling.
    commands.entity(e).insert(DespawnOnExit(Screen::Title));
    commands.queue(move |world: &mut World| {
        if let Some(mut b) = world.get_mut::<MsgBoxWidget>(e) {
            b.active = false;
        }
    });
}

fn spawn_start_menu(mut commands: Commands, font: Res<UiFont>) {
    let labels = START_ITEMS.iter().map(|(l, _)| encode(l)).collect();
    let e = spawn_list(
        &mut commands,
        &font,
        LAYER_BOTTOM,
        (140.0, 12.0),
        labels,
        5,
        true,
    );
    commands
        .entity(e)
        .insert((DespawnOnExit(Overlay::StartMenu), StartMenuList));
}

/// Marks the start-menu list (the yes/no list must not open overlays).
#[derive(Component)]
struct StartMenuList;

fn start_menu_events(
    lists: Query<&ListWidget, With<StartMenuList>>,
    mut next: Option<ResMut<NextState<Overlay>>>,
) {
    use super::list::ListEvent;
    for l in &lists {
        if let (ListEvent::Select(i), Some(next)) = (l.event, next.as_mut()) {
            if let Some((_, o)) = START_ITEMS.get(i) {
                next.set(*o);
            }
        }
    }
}

const MSG_POS: (f32, f32) = (8.0, 136.0);

#[allow(clippy::too_many_arguments)] // Bevy system parameters
fn dialog_demo(
    mut commands: Commands,
    mut stage: ResMut<Stage>,
    input: Res<UiInput>,
    font: Res<UiFont>,
    settings: Res<UiSettings>,
    screen: Res<State<Screen>>,
    overlay: Option<Res<State<Overlay>>>,
    mut boxes: Query<&mut MsgBoxWidget>,
    lists: Query<&ListWidget>,
    numbers: Query<&NumberWidget>,
) {
    let msg = |commands: &mut Commands, text: &str| {
        spawn_msgbox(
            commands,
            &font,
            &settings,
            LAYER_TOP,
            MSG_POS,
            &encode(text),
        )
    };
    let finished =
        |boxes: &Query<&mut MsgBoxWidget>, e: Entity| boxes.get(e).is_ok_and(|b| b.finished);
    match &mut *stage {
        Stage::Idle => {
            let open = *screen.get() == Screen::Overworld
                && overlay.is_none_or(|o| *o.get() == Overlay::Closed);
            if open && input.y {
                let e = msg(
                    &mut commands,
                    "This is the G5 UI kit.\nText appears letter by letter.\x0cA or B turns the page, or\nshows the rest at once.\x0cLet's try a few widgets!",
                );
                *stage = Stage::Intro(e);
            }
        }
        Stage::Intro(e) => {
            if finished(&boxes, *e) {
                commands.entity(*e).despawn();
                let m = msg(&mut commands, "Do you like it?");
                // Read-only: only the yes/no box takes A/B.
                commands.queue(move |world: &mut World| {
                    if let Some(mut b) = world.get_mut::<MsgBoxWidget>(m) {
                        b.active = false;
                    }
                });
                *stage = Stage::Ask { msg: m, list: None };
            }
        }
        Stage::Ask { msg: m, list } => match list {
            None => {
                // Show the yes/no box once the question is fully typed.
                if let Ok(mut b) = boxes.get_mut(*m) {
                    if b.logic.arrow() {
                        b.active = false;
                        let l = spawn_list(
                            &mut commands,
                            &font,
                            LAYER_TOP,
                            (192.0, 84.0),
                            vec![encode("YES"), encode("NO")],
                            2,
                            false,
                        );
                        *list = Some(l);
                    }
                }
            }
            Some(l) => {
                let Ok(w) = lists.get(*l) else { return };
                if let Some(yes) = yes_no_answer(w.event) {
                    commands.entity(*l).despawn();
                    commands.entity(*m).despawn();
                    *stage = if yes {
                        let m = msg(&mut commands, "How many would you like?");
                        let n = spawn_number(
                            &mut commands,
                            LAYER_TOP,
                            (196.0, 104.0),
                            NumberInput::new(3, 1, 999, 1),
                        );
                        // The prompt is read-only while the number box has focus.
                        commands.queue(move |world: &mut World| {
                            if let Some(mut b) = world.get_mut::<MsgBoxWidget>(m) {
                                b.active = false;
                            }
                        });
                        Stage::Count { msg: m, num: n }
                    } else {
                        Stage::Outro(msg(&mut commands, "Oh... maybe next time."))
                    };
                }
            }
        },
        Stage::Count { msg: m, num } => {
            let Ok(w) = numbers.get(*num) else { return };
            let text = match w.event {
                NumberEvent::Done(n) => Some(format!("You picked {n}!")),
                NumberEvent::Cancel => Some("Cancelled.".to_string()),
                _ => None,
            };
            if let Some(t) = text {
                commands.entity(*num).despawn();
                commands.entity(*m).despawn();
                *stage = Stage::Outro(msg(&mut commands, &t));
            }
        }
        Stage::Outro(e) => {
            if finished(&boxes, *e) {
                commands.entity(*e).despawn();
                *stage = Stage::Idle;
            }
        }
    }
}
