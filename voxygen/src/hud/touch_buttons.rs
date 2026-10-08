//! On-screen touch buttons for phones (Android), drawn over the game while the
//! pointer is grabbed.
//!
//! Every button is described by the same data. The HUD draws them and the
//! session hit-tests the same regions, so the two cannot drift apart.
//!
//! - Fixed buttons (movement, combat, skill slots and menus) are always shown.
//! - Context buttons are shown only while [`layout`] offers their action, so
//!   they appear and disappear with the state (e.g. Glide only with a glider).
//!   Less common actions sit behind a "More" button, so the screen does not
//!   fill up with buttons.

use crate::game_input::GameInput;
use vek::Vec2;

/// What a button does when tapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Sends the input exactly like pressing its key.
    Input(GameInput),
    /// Shows or hides the secondary row.
    More,
}

/// Facts about the player that decide which actions are offered.
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    /// The player has a living character that can take actions.
    pub controlling: bool,
    pub riding: bool,
    pub wielding: bool,
    pub gliding: bool,
    /// A glider is equipped, so gliding can start.
    pub has_glider: bool,
    /// A lantern is equipped, so the lantern can be turned on.
    pub has_lantern: bool,
    pub lantern_on: bool,
    pub sneaking: bool,
    pub sitting: bool,
    pub crawling: bool,
    pub dancing: bool,
    pub zoom_locked: bool,
}

/// Describes one button before its text is localised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub kind: Kind,
    /// Fluent message id for the button name.
    pub label_key: &'static str,
    /// Shows the button highlighted, e.g. while gliding.
    pub active: bool,
    /// Disabled buttons are not offered.
    pub enabled: bool,
}

impl Spec {
    fn input(input: GameInput, label_key: &'static str, active: bool, enabled: bool) -> Self {
        Self {
            kind: Kind::Input(input),
            label_key,
            active,
            enabled,
        }
    }
}

/// Returns the primary and secondary actions for the given state.
///
/// The secondary actions are empty unless `expanded` is set.
pub fn layout(ctx: &Context, expanded: bool) -> (Vec<Spec>, Vec<Spec>) {
    let mut primary = Vec::new();
    let mut secondary = Vec::new();

    if ctx.controlling && !ctx.riding {
        if ctx.gliding {
            primary.push(Spec::input(GameInput::Glide, "gameinput-glide", true, true));
        } else {
            primary.push(Spec::input(
                GameInput::ToggleWield,
                if ctx.wielding {
                    "hud-action-sheathe"
                } else {
                    "hud-action-draw"
                },
                ctx.wielding,
                true,
            ));
            if ctx.wielding {
                primary.push(Spec::input(GameInput::Block, "gameinput-block", false, true));
            }
            primary.push(Spec::input(GameInput::Roll, "gameinput-roll", false, true));
            primary.push(Spec::input(GameInput::Jump, "gameinput-jump", false, true));
            primary.push(Spec::input(
                GameInput::Glide,
                "gameinput-glide",
                false,
                ctx.has_glider,
            ));
        }
        primary.push(Spec::input(
            GameInput::ToggleLantern,
            "gameinput-togglelantern",
            ctx.lantern_on,
            ctx.has_lantern || ctx.lantern_on,
        ));
    }

    primary.push(Spec::input(GameInput::ZoomIn, "gameinput-zoomin", false, true));
    primary.push(Spec::input(GameInput::ZoomOut, "gameinput-zoomout", false, true));

    if ctx.controlling && !ctx.riding && !ctx.gliding {
        secondary.push(Spec::input(
            GameInput::Sneak,
            "gameinput-sneak",
            ctx.sneaking,
            true,
        ));
        secondary.push(Spec::input(GameInput::Sit, "gameinput-sit", ctx.sitting, true));
        secondary.push(Spec::input(
            GameInput::Crawl,
            "gameinput-crawl",
            ctx.crawling,
            true,
        ));
        secondary.push(Spec::input(
            GameInput::Dance,
            "gameinput-dance",
            ctx.dancing,
            true,
        ));
        secondary.push(Spec::input(
            GameInput::Greet,
            "gameinput-greet",
            false,
            true,
        ));
    }
    secondary.push(Spec::input(
        GameInput::ZoomLock,
        "gameinput-zoomlock",
        ctx.zoom_locked,
        true,
    ));

    primary.push(Spec {
        kind: Kind::More,
        label_key: if expanded {
            "hud-action-less"
        } else {
            "hud-action-more"
        },
        active: expanded,
        enabled: true,
    });

    let secondary = if expanded { secondary } else { Vec::new() };
    (primary, secondary)
}

/// Diameter of the round context buttons, as a fraction of the screen height.
const CONTEXT_DIAMETER: f32 = 0.15;

/// The text on a button.
#[derive(Clone, Copy)]
enum Label {
    /// Fluent message id.
    Key(&'static str),
    /// A symbol that needs no translation.
    Symbol(&'static str),
}

/// Always-visible buttons: `(action, label, x, y, diameter)`.
///
/// Positions are fractions of the screen: `x` of the width, `y` of the height
/// measured from the top, and `diameter` of the height. The movement stick has
/// no action; the session handles the left half of the screen itself.
const FIXED: [(Option<Kind>, Label, f32, f32, f32); 15] = [
    (None, Label::Key("hud-touch-move"), 0.20, 0.76, 0.34),
    (
        Some(Kind::Input(GameInput::Primary)),
        Label::Key("gameinput-primary"),
        0.90,
        0.78,
        0.15,
    ),
    (
        Some(Kind::Input(GameInput::Secondary)),
        Label::Key("hud-touch-alt"),
        0.76,
        0.85,
        0.15,
    ),
    (
        Some(Kind::Input(GameInput::Jump)),
        Label::Key("gameinput-jump"),
        0.76,
        0.66,
        0.15,
    ),
    (
        Some(Kind::Input(GameInput::Interact)),
        Label::Key("gameinput-interact"),
        0.90,
        0.57,
        0.15,
    ),
    (
        Some(Kind::Input(GameInput::Roll)),
        Label::Key("gameinput-roll"),
        0.76,
        0.47,
        0.15,
    ),
    (
        Some(Kind::Input(GameInput::Slot1)),
        Label::Symbol("1"),
        0.50,
        0.26,
        0.10,
    ),
    (
        Some(Kind::Input(GameInput::Slot2)),
        Label::Symbol("2"),
        0.60,
        0.26,
        0.10,
    ),
    (
        Some(Kind::Input(GameInput::Slot3)),
        Label::Symbol("3"),
        0.70,
        0.26,
        0.10,
    ),
    (
        Some(Kind::Input(GameInput::Slot4)),
        Label::Symbol("4"),
        0.80,
        0.26,
        0.10,
    ),
    (
        Some(Kind::Input(GameInput::Slot5)),
        Label::Symbol("5"),
        0.90,
        0.26,
        0.10,
    ),
    (
        Some(Kind::Input(GameInput::Inventory)),
        Label::Key("gameinput-inventory"),
        0.56,
        0.075,
        0.09,
    ),
    (
        Some(Kind::Input(GameInput::Diary)),
        Label::Key("gameinput-diary"),
        0.66,
        0.075,
        0.09,
    ),
    (
        Some(Kind::Input(GameInput::Settings)),
        Label::Key("gameinput-settings"),
        0.76,
        0.075,
        0.09,
    ),
    (
        Some(Kind::Input(GameInput::Escape)),
        Label::Key("hud-touch-menu"),
        0.86,
        0.075,
        0.09,
    ),
];

/// Primary context actions: `(input, x, y)`. Each has its own slot, so a button
/// never moves when another one appears or disappears.
const PRIMARY: [(GameInput, f32, f32); 6] = [
    (GameInput::ToggleWield, 0.66, 0.47),
    (GameInput::Block, 0.56, 0.47),
    (GameInput::Glide, 0.66, 0.66),
    (GameInput::ToggleLantern, 0.66, 0.85),
    (GameInput::ZoomIn, 0.56, 0.66),
    (GameInput::ZoomOut, 0.56, 0.85),
];

/// The "More" button, which shows the secondary row.
const MORE: (f32, f32) = (0.90, 0.415);

/// Secondary context actions, shown only while "More" is expanded.
const SECONDARY: [(GameInput, f32, f32); 6] = [
    (GameInput::Sneak, 0.46, 0.47),
    (GameInput::Sit, 0.46, 0.66),
    (GameInput::Crawl, 0.46, 0.85),
    (GameInput::Dance, 0.36, 0.47),
    (GameInput::Greet, 0.36, 0.66),
    (GameInput::ZoomLock, 0.36, 0.85),
];

const PRIMARY_BASE: usize = FIXED.len();
const MORE_INDEX: usize = PRIMARY_BASE + PRIMARY.len();
const SECONDARY_BASE: usize = MORE_INDEX + 1;

/// Total number of button slots; used to size the widget ids.
pub const COUNT: usize = SECONDARY_BASE + SECONDARY.len();

/// A button to draw this frame, with its label already localised.
pub struct Shown {
    /// Stable slot index, used to pick the widget ids.
    pub index: usize,
    /// `None` for purely visual buttons (the movement stick).
    pub action: Option<Kind>,
    /// Centre as fractions of the screen (x of width, y of height from top).
    pub center: Vec2<f32>,
    /// Diameter as a fraction of the screen height.
    pub diameter: f32,
    pub label: String,
    /// Drawn highlighted, e.g. while gliding or while "More" is expanded.
    pub active: bool,
}

/// A region the session can hit-test, in the same fractions as [`Shown`].
pub struct Region {
    pub action: Kind,
    pub center: Vec2<f32>,
    /// Radius as a fraction of the screen height.
    pub radius: f32,
}

/// Returns the buttons to show for the current state.
///
/// `expanded` is the state of the "More" button.
pub fn shown(ctx: &Context, expanded: bool, localize: impl Fn(&str) -> String) -> Vec<Shown> {
    let mut shown: Vec<Shown> = FIXED
        .iter()
        .enumerate()
        .map(|(index, &(action, label, x, y, diameter))| Shown {
            index,
            action,
            center: Vec2::new(x, y),
            diameter,
            label: match label {
                Label::Key(key) => localize(key),
                Label::Symbol(symbol) => symbol.to_string(),
            },
            active: false,
        })
        .collect();

    let (primary, secondary) = layout(ctx, expanded);
    for spec in primary.iter().chain(&secondary) {
        if let Some((index, x, y)) = slot_for(spec, ctx) {
            shown.push(Shown {
                index,
                action: Some(spec.kind),
                center: Vec2::new(x, y),
                diameter: CONTEXT_DIAMETER,
                label: localize(spec.label_key),
                active: spec.active,
            });
        }
    }

    shown
}

/// Returns the slot `(index, x, y)` for an action that should be shown now.
fn slot_for(spec: &Spec, ctx: &Context) -> Option<(usize, f32, f32)> {
    // Unavailable actions are hidden rather than dimmed: this overlay only
    // offers what applies right now.
    if !spec.enabled {
        return None;
    }
    match spec.kind {
        Kind::More => Some((MORE_INDEX, MORE.0, MORE.1)),
        Kind::Input(input) => {
            // The zoom buttons do nothing while the camera zoom is locked.
            if ctx.zoom_locked && matches!(input, GameInput::ZoomIn | GameInput::ZoomOut) {
                return None;
            }
            PRIMARY
                .iter()
                .enumerate()
                .find(|(_, slot)| slot.0 == input)
                .map(|(n, slot)| (PRIMARY_BASE + n, slot.1, slot.2))
                .or_else(|| {
                    SECONDARY
                        .iter()
                        .enumerate()
                        .find(|(_, slot)| slot.0 == input)
                        .map(|(n, slot)| (SECONDARY_BASE + n, slot.1, slot.2))
                })
        },
    }
}

/// Returns the regions that the session can tap.
pub fn regions(shown: &[Shown]) -> Vec<Region> {
    shown
        .iter()
        .filter_map(|button| {
            button.action.map(|action| Region {
                action,
                center: button.center,
                radius: button.diameter / 2.0,
            })
        })
        .collect()
}
