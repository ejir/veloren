//! On-screen touch buttons for phones, drawn over the game while the pointer is
//! grabbed.
//!
//! Every button is described by the same data, and the HUD draws them and the
//! session hit-tests them from the same list, so the two cannot drift apart.
//!
//! Two kinds of button exist:
//! - Fixed buttons (combat, skill slots and menus) are always shown.
//! - Context buttons have a fixed position, and are shown only while the action
//!   bar offers their action. They reuse [`action_bar::layout`], so the rules
//!   (e.g. "Glide only with a glider", "Block only while wielding") are the same
//!   as on the action bar, and their labels come from the same i18n keys.

use super::action_bar::{self, Kind, Spec};
use crate::game_input::GameInput;
use vek::Vec2;

/// Diameter of the round context buttons, as a fraction of the screen height.
const CONTEXT_DIAMETER: f32 = 0.15;

/// Always-visible buttons: `(input, label, x, y, diameter)`.
///
/// Positions are fractions of the screen: `x` of the width, `y` of the height
/// measured from the top, and `diameter` of the height. The movement stick has
/// no input here; the session handles the left half of the screen itself.
const FIXED: [(Option<GameInput>, &str, f32, f32, f32); 15] = [
    (None, "MOVE", 0.20, 0.76, 0.34),
    (Some(GameInput::Primary), "ATK", 0.90, 0.78, 0.15),
    (Some(GameInput::Secondary), "ALT", 0.76, 0.85, 0.15),
    (Some(GameInput::Jump), "JUMP", 0.76, 0.66, 0.15),
    (Some(GameInput::Interact), "USE", 0.90, 0.57, 0.15),
    (Some(GameInput::Roll), "ROLL", 0.76, 0.47, 0.15),
    (Some(GameInput::Slot1), "1", 0.50, 0.26, 0.10),
    (Some(GameInput::Slot2), "2", 0.60, 0.26, 0.10),
    (Some(GameInput::Slot3), "3", 0.70, 0.26, 0.10),
    (Some(GameInput::Slot4), "4", 0.80, 0.26, 0.10),
    (Some(GameInput::Slot5), "5", 0.90, 0.26, 0.10),
    (Some(GameInput::Inventory), "BAG", 0.56, 0.075, 0.09),
    (Some(GameInput::Diary), "SKILL", 0.66, 0.075, 0.09),
    (Some(GameInput::Settings), "SET", 0.76, 0.075, 0.09),
    (Some(GameInput::Escape), "MENU", 0.86, 0.075, 0.09),
];

/// Context buttons: `(input, x, y)`. Each one has its own slot, so a button
/// never moves when another one appears or disappears.
const CONTEXT: [(GameInput, f32, f32); 7] = [
    (GameInput::ToggleWield, 0.66, 0.47),
    (GameInput::Glide, 0.66, 0.66),
    (GameInput::ToggleLantern, 0.66, 0.85),
    (GameInput::Block, 0.56, 0.47),
    (GameInput::Sneak, 0.56, 0.66),
    (GameInput::Sit, 0.56, 0.85),
    (GameInput::Greet, 0.90, 0.415),
];

/// Total number of button slots; used to size the widget ids.
pub const COUNT: usize = FIXED.len() + CONTEXT.len();

/// A button to draw this frame, with its label already localised.
pub struct Shown {
    /// Stable slot index, used to pick the widget ids.
    pub index: usize,
    pub input: Option<GameInput>,
    /// Centre as fractions of the screen (x of width, y of height from top).
    pub center: Vec2<f32>,
    /// Diameter as a fraction of the screen height.
    pub diameter: f32,
    pub label: String,
    /// Drawn highlighted, e.g. while gliding.
    pub active: bool,
    /// Disabled buttons are drawn dimmed and do not send input.
    pub enabled: bool,
}

/// A region the session can hit-test, in the same fractions as [`Shown`].
pub struct Region {
    pub input: GameInput,
    pub center: Vec2<f32>,
    /// Radius as a fraction of the screen height.
    pub radius: f32,
}

/// Returns the buttons to show for the current state.
pub fn shown(ctx: &action_bar::Context, localize: impl Fn(&str) -> String) -> Vec<Shown> {
    // Both rows, so context buttons behind "More" on the action bar are offered too.
    let (primary, secondary) = action_bar::layout(ctx, true);
    let specs: Vec<Spec> = primary.into_iter().chain(secondary).collect();

    let mut shown: Vec<Shown> = FIXED
        .iter()
        .enumerate()
        .map(|(index, &(input, label, x, y, diameter))| Shown {
            index,
            input,
            center: Vec2::new(x, y),
            diameter,
            label: label.to_string(),
            active: false,
            enabled: true,
        })
        .collect();

    for (slot, &(input, x, y)) in CONTEXT.iter().enumerate() {
        let Some(spec) = specs.iter().find(|spec| spec.kind == Kind::Input(input)) else {
            continue;
        };
        shown.push(Shown {
            index: FIXED.len() + slot,
            input: Some(input),
            center: Vec2::new(x, y),
            diameter: CONTEXT_DIAMETER,
            label: localize(spec.label_key),
            active: spec.active,
            enabled: spec.enabled,
        });
    }

    shown
}

/// Returns the regions that send input. Disabled buttons send nothing.
pub fn regions(shown: &[Shown]) -> Vec<Region> {
    shown
        .iter()
        .filter(|button| button.enabled)
        .filter_map(|button| {
            button.input.map(|input| Region {
                input,
                center: button.center,
                radius: button.diameter / 2.0,
            })
        })
        .collect()
}
