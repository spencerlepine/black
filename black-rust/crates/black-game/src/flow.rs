//! Pure app-flow logic (G2): screen/overlay enums, the debug transition table and
//! the DS-style brightness fade. No window or ECS access here, so it is unit-tested directly.

use bevy::prelude::{Resource, States, SubStates};

use crate::input::DsButton;

/// Top-level screen, mirroring the original proc/overlay flow at a high level.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Screen {
    /// System/FS init, first-boot logo (original: boot + gfl_use init).
    #[default]
    Boot,
    Title,
    NewGame,
    Continue,
    Overworld,
    Battle,
}

/// Overlay menu on top of the overworld (a substate of `Screen::Overworld`).
#[derive(SubStates, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[source(Screen = Screen::Overworld)]
pub enum Overlay {
    #[default]
    Closed,
    StartMenu,
    Party,
    Bag,
    Summary,
    PCBox,
    Pokedex,
    Save,
}

impl Overlay {
    /// Where B goes from this overlay: sub-menus return to the start menu.
    pub fn parent(self) -> Overlay {
        match self {
            Overlay::Closed | Overlay::StartMenu => Overlay::Closed,
            _ => Overlay::StartMenu,
        }
    }
}

/// What a button press asks the app to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Change screen (goes through a fade).
    Screen(Screen),
    /// Change overlay (instant, like the original menu open/close).
    Overlay(Overlay),
}

/// Debug transition table: what `button` does on `screen` with `overlay` open.
pub fn on_button(screen: Screen, overlay: Overlay, button: DsButton) -> Option<Request> {
    use DsButton as B;
    use Request::{Overlay as O, Screen as S};
    match (screen, overlay, button) {
        (Screen::Title, _, B::A) => Some(S(Screen::NewGame)),
        (Screen::Title, _, B::Y) => Some(S(Screen::Continue)),
        (Screen::Overworld, Overlay::Closed, B::Start) => Some(O(Overlay::StartMenu)),
        (Screen::Overworld, Overlay::Closed, B::X) => Some(S(Screen::Battle)),
        (Screen::Overworld, Overlay::StartMenu, B::Start) => Some(O(Overlay::Closed)),
        (Screen::Overworld, Overlay::StartMenu, B::L) => Some(O(Overlay::Party)),
        (Screen::Overworld, Overlay::StartMenu, B::R) => Some(O(Overlay::Bag)),
        (Screen::Overworld, Overlay::StartMenu, B::X) => Some(O(Overlay::Summary)),
        (Screen::Overworld, Overlay::StartMenu, B::Y) => Some(O(Overlay::PCBox)),
        (Screen::Overworld, Overlay::StartMenu, B::Select) => Some(O(Overlay::Pokedex)),
        // A is not mapped: the start-menu list widget (ui::demo) takes A.
        (Screen::Overworld, o, B::B) if o != Overlay::Closed => Some(O(o.parent())),
        (Screen::Battle, _, B::A) => Some(S(Screen::Overworld)),
        _ => None,
    }
}

/// Screen a placeholder advances to by itself once its dwell time elapses.
pub fn on_auto(screen: Screen) -> Option<Screen> {
    match screen {
        Screen::Boot => Some(Screen::Title),
        Screen::NewGame | Screen::Continue => Some(Screen::Overworld),
        _ => None,
    }
}

/// Number of DS master-brightness steps (0 = clear, 16 = black).
pub const FADE_LEVELS: u32 = 16;

/// Darkness level (0..=16) `frame` frames into a fade-to-black lasting `total` frames.
/// A zero-length fade is fully dark.
pub fn darkness(frame: u32, total: u32) -> u32 {
    if total == 0 {
        return FADE_LEVELS;
    }
    (frame.min(total) * FADE_LEVELS + total / 2) / total
}

/// Durations (in 60 Hz frames) for fades and placeholder dwell times.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FlowConfig {
    pub fade_out_frames: u32,
    pub fade_in_frames: u32,
    /// Boot screen dwell before moving to the title.
    pub boot_frames: u32,
    /// NewGame/Continue placeholder dwell before the overworld.
    pub load_frames: u32,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            fade_out_frames: 16,
            fade_in_frames: 16,
            boot_frames: 30,
            load_frames: 30,
        }
    }
}

impl FlowConfig {
    /// Dwell time of `screen` before its automatic advance.
    pub fn dwell(&self, screen: Screen) -> u32 {
        match screen {
            Screen::Boot => self.boot_frames,
            _ => self.load_frames,
        }
    }
}

/// Fade state machine, ticked once per 60 Hz frame.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    Idle,
    /// Going to black; `then` is applied when fully black.
    Out {
        frame: u32,
        total: u32,
        then: Request,
    },
    /// Coming back from black.
    In {
        frame: u32,
        total: u32,
    },
}

/// Result of one fade tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeEvent {
    None,
    /// Screen is fully black: apply this request now.
    Apply(Request),
}

impl Fade {
    /// Begin fading out to apply `then`.
    pub fn start(then: Request, cfg: &FlowConfig) -> Fade {
        Fade::Out {
            frame: 0,
            total: cfg.fade_out_frames,
            then,
        }
    }

    /// Current darkness 0..=16.
    pub fn darkness(&self) -> u32 {
        match *self {
            Fade::Idle => 0,
            Fade::Out { frame, total, .. } => darkness(frame, total),
            Fade::In { frame, total } => FADE_LEVELS - darkness(frame, total),
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Fade::Idle)
    }

    /// Advance one frame.
    pub fn tick(&mut self, cfg: &FlowConfig) -> FadeEvent {
        match *self {
            Fade::Idle => FadeEvent::None,
            Fade::Out { frame, total, then } => {
                if frame >= total {
                    *self = Fade::In {
                        frame: 0,
                        total: cfg.fade_in_frames,
                    };
                    FadeEvent::Apply(then)
                } else {
                    *self = Fade::Out {
                        frame: frame + 1,
                        total,
                        then,
                    };
                    FadeEvent::None
                }
            }
            Fade::In { frame, total } => {
                *self = if frame >= total {
                    Fade::Idle
                } else {
                    Fade::In {
                        frame: frame + 1,
                        total,
                    }
                };
                FadeEvent::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn darkness_curve() {
        assert_eq!(darkness(0, 16), 0);
        assert_eq!(darkness(8, 16), 8);
        assert_eq!(darkness(16, 16), 16);
        assert_eq!(darkness(99, 16), 16);
        assert_eq!(darkness(0, 0), 16);
        // Monotonic for an awkward length.
        let mut last = 0;
        for f in 0..=30 {
            let d = darkness(f, 30);
            assert!(d >= last && d <= FADE_LEVELS);
            last = d;
        }
        assert_eq!(last, 16);
    }

    #[test]
    fn fade_sequence() {
        let cfg = FlowConfig {
            fade_out_frames: 4,
            fade_in_frames: 4,
            ..FlowConfig::default()
        };
        let req = Request::Screen(Screen::Title);
        let mut f = Fade::start(req, &cfg);
        let mut applied_at = None;
        let mut levels = vec![f.darkness()];
        for i in 0..20 {
            if f.tick(&cfg) == FadeEvent::Apply(req) {
                applied_at = Some(i);
                assert_eq!(f.darkness(), FADE_LEVELS);
            }
            levels.push(f.darkness());
            if f.is_idle() {
                break;
            }
        }
        assert_eq!(applied_at, Some(4));
        assert_eq!(levels, [0, 4, 8, 12, 16, 16, 12, 8, 4, 0, 0]);
        assert!(f.is_idle());
    }

    #[test]
    fn zero_length_fade_still_applies() {
        let cfg = FlowConfig {
            fade_out_frames: 0,
            fade_in_frames: 0,
            ..FlowConfig::default()
        };
        let req = Request::Screen(Screen::Battle);
        let mut f = Fade::start(req, &cfg);
        assert_eq!(f.tick(&cfg), FadeEvent::Apply(req));
        f.tick(&cfg);
        f.tick(&cfg);
        assert!(f.is_idle());
    }

    #[test]
    fn table_title_and_battle() {
        use DsButton as B;
        let c = Overlay::Closed;
        assert_eq!(
            on_button(Screen::Title, c, B::A),
            Some(Request::Screen(Screen::NewGame))
        );
        assert_eq!(
            on_button(Screen::Title, c, B::Y),
            Some(Request::Screen(Screen::Continue))
        );
        assert_eq!(on_button(Screen::Title, c, B::B), None);
        assert_eq!(
            on_button(Screen::Overworld, c, B::X),
            Some(Request::Screen(Screen::Battle))
        );
        assert_eq!(
            on_button(Screen::Battle, c, B::A),
            Some(Request::Screen(Screen::Overworld))
        );
        assert_eq!(on_button(Screen::Boot, c, B::A), None);
    }

    #[test]
    fn table_overlays() {
        use DsButton as B;
        let sm = Overlay::StartMenu;
        assert_eq!(
            on_button(Screen::Overworld, Overlay::Closed, B::Start),
            Some(Request::Overlay(sm))
        );
        // B closes the start menu; X in the menu opens Summary, not Battle.
        assert_eq!(
            on_button(Screen::Overworld, sm, B::B),
            Some(Request::Overlay(Overlay::Closed))
        );
        assert_eq!(
            on_button(Screen::Overworld, sm, B::X),
            Some(Request::Overlay(Overlay::Summary))
        );
        // Every sub-menu is reachable and B returns to the start menu.
        for (b, o) in [
            (B::L, Overlay::Party),
            (B::R, Overlay::Bag),
            (B::X, Overlay::Summary),
            (B::Y, Overlay::PCBox),
            (B::Select, Overlay::Pokedex),
        ] {
            assert_eq!(
                on_button(Screen::Overworld, sm, b),
                Some(Request::Overlay(o))
            );
            assert_eq!(
                on_button(Screen::Overworld, o, B::B),
                Some(Request::Overlay(sm))
            );
        }
        // No overlays outside the overworld.
        assert_eq!(on_button(Screen::Battle, Overlay::Closed, B::Start), None);
    }

    #[test]
    fn auto_advance() {
        assert_eq!(on_auto(Screen::Boot), Some(Screen::Title));
        assert_eq!(on_auto(Screen::NewGame), Some(Screen::Overworld));
        assert_eq!(on_auto(Screen::Continue), Some(Screen::Overworld));
        assert_eq!(on_auto(Screen::Title), None);
        assert_eq!(on_auto(Screen::Battle), None);
    }
}
