//! On-screen touch buttons for phones (Android), drawn over the game while the
//! pointer is grabbed.
//!
//! Every button is described by the same data: the HUD draws it and the session
//! hit-tests the same region, so the two cannot drift apart.
//!
//! # Layout
//!
//! Diameters and rows are fractions of the screen *height*, and the right-hand
//! zones are lanes counted inwards from the right edge. One unit therefore
//! means the same physical size on a 4:3 tablet and on a 21:9 phone. Callers
//! pass the aspect ratio; [`Shown::center`] comes back as fractions of the
//! screen, which is what the drawing code and the hit-testing already speak.
//!
//! The screen is organised in rows (menus, skill slots, then the combat
//! cluster) and lanes (edge, inner, context). Two rules keep it from getting
//! crowded:
//!
//! - The context actions form a single list. While "More" is closed, only the
//!   first [`TIER_SLOTS`] of them are offered, in a column beside the combat
//!   cluster. Opening "More" swaps that column for a grid with room for every
//!   context action, so the screen never carries two piles of them at once.
//! - The grid is measured out from the space actually left between the movement
//!   stick and the combat cluster ([`grid`]), so it fits narrow screens instead
//!   of running into either: it drops columns and shrinks its buttons rather
//!   than letting them touch.
//!
//! The tests at the bottom assert the invariants for every device shape we care
//! about: nothing overlaps and nothing leaves the screen.

use crate::game_input::GameInput;
use vek::Vec2;

/// What a button does when tapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Sends the input exactly like pressing its key.
    Input(GameInput),
    /// Shows or hides the context grid.
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

/// Describes one context action before its text is localised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub kind: Kind,
    /// Fluent message id for the button name.
    pub label_key: &'static str,
    /// Shows the button highlighted, e.g. while gliding.
    pub active: bool,
    /// Disabled buttons are not offered.
    pub enabled: bool,
    /// Place in the context list; also picks the position. Slots below
    /// [`TIER_SLOTS`] are offered while "More" is closed.
    pub slot: usize,
}

/// The context actions, in slot order. Slot order is the reading order of the
/// grid, so the actions that belong together share a row: combat, then the
/// toggles and emotes, then the camera.
///
/// Whether an action is offered at all comes from [`state_of`], which keeps
/// this a plain list.
const ACTIONS: [(GameInput, &str); CONTEXT_SLOTS] = [
    (GameInput::ToggleWield, "hud-touch-draw"),
    (GameInput::Block, "hud-touch-block"),
    (GameInput::Glide, "hud-touch-glide"),
    (GameInput::ToggleLantern, "hud-touch-lantern"),
    (GameInput::Sneak, "hud-touch-sneak"),
    (GameInput::Sit, "hud-touch-sit"),
    (GameInput::Crawl, "hud-touch-crawl"),
    (GameInput::Dance, "hud-touch-dance"),
    (GameInput::Greet, "hud-touch-greet"),
    (GameInput::ZoomIn, "hud-touch-zoomin"),
    (GameInput::ZoomOut, "hud-touch-zoomout"),
    (GameInput::ZoomLock, "hud-touch-zoomlock"),
];

/// Returns every context action, in slot order.
pub fn layout(ctx: &Context) -> Vec<Spec> {
    ACTIONS
        .into_iter()
        .enumerate()
        .map(|(slot, (input, label_key))| {
            let (active, enabled) = state_of(ctx, input);
            // Drawing and sheathing are one input with two names.
            let label_key = match input {
                GameInput::ToggleWield if ctx.wielding => "hud-touch-sheathe",
                _ => label_key,
            };
            Spec {
                kind: Kind::Input(input),
                label_key,
                active,
                enabled,
                slot,
            }
        })
        .collect()
}

/// Whether an action is lit up, and whether the current state offers it.
fn state_of(ctx: &Context, input: GameInput) -> (bool, bool) {
    let ready = ctx.controlling && !ctx.riding;
    let still = ready && !ctx.gliding;
    let glider = ctx.gliding || ctx.has_glider;
    let lantern = ctx.has_lantern || ctx.lantern_on;
    match input {
        GameInput::ToggleWield => (ctx.wielding, ready),
        GameInput::Block => (false, still && ctx.wielding),
        GameInput::Glide => (ctx.gliding, ready && glider),
        GameInput::ToggleLantern => (ctx.lantern_on, ready && lantern),
        GameInput::Sneak => (ctx.sneaking, still),
        GameInput::Sit => (ctx.sitting, still),
        GameInput::Crawl => (ctx.crawling, still),
        GameInput::Dance => (ctx.dancing, still),
        GameInput::Greet => (false, still),
        // Zooming does nothing while the zoom is locked, so hide it then.
        GameInput::ZoomIn | GameInput::ZoomOut => (false, !ctx.zoom_locked),
        GameInput::ZoomLock => (ctx.zoom_locked, ready),
        _ => (false, false),
    }
}

/// Diameters, as fractions of the screen height.
const D_STICK: f32 = 0.30;
const D_ATTACK: f32 = 0.18;
const D_SMALL: f32 = 0.13;
const D_SLOT: f32 = 0.095;
const D_MENU: f32 = 0.085;

/// Rows, as fractions of the screen height measured from the top.
const ROW_MENU: f32 = 0.055;
const ROW_SLOT: f32 = 0.175;
const ROW_HIGH: f32 = 0.395;
const ROW_MID: f32 = 0.585;
const ROW_LOW: f32 = 0.80;

/// Lanes, in screen-height units measured inwards from the right edge.
const LANE_EDGE: f32 = 0.13;
const LANE_INNER: f32 = 0.345;
/// Where the context column sits while "More" is closed.
const LANE_CONTEXT: f32 = 0.56;
const LANE_MORE: f32 = 0.775;

/// Lanes of the five skill slots, and of the three menu buttons. Neighbours are
/// one diameter plus a 0.03 gap apart, which is as tight as a thumb wants.
const SLOT_LANES: [f32; 5] = [0.13, 0.255, 0.38, 0.505, 0.63];
const MENU_LANES: [f32; 3] = [0.13, 0.245, 0.36];

/// The movement stick is placed by width instead, so it stays under the left
/// thumb however wide the screen is.
const STICK_X: f32 = 0.16;
const STICK_Y: f32 = 0.73;

/// Most columns the context grid uses, on a screen wide enough for them.
const PANEL_COLS: usize = 3;
/// Fewest columns it falls back to. A single column of twelve would not fit
/// vertically either, so below this the buttons shrink instead.
const PANEL_COLS_MIN: usize = 2;
/// Keep-out between the grid and whatever is beside it.
const PANEL_MARGIN: f32 = 0.02;
/// Gap the grid keeps between its own buttons, as for the rest of the layout.
const BUTTON_GAP: f32 = 0.03;
/// Widest the grid is allowed to stretch on a very wide screen.
const PANEL_SPACING_MAX: f32 = 0.30;
/// Smallest a grid button shrinks to. Only reached in windows narrower than
/// anything the Android build can be given.
const PANEL_D_MIN: f32 = 0.06;
/// First row of the grid, how far down it may reach, and the row pitch it
/// prefers when there is room for it.
const PANEL_ROW_TOP: f32 = 0.36;
const PANEL_ROW_BOTTOM: f32 = 0.92;
const PANEL_ROW_PITCH: f32 = 0.16;

/// Context actions offered while "More" is closed, and where they sit.
const TIER_SLOTS: usize = 3;
const TIER_ROWS: [f32; TIER_SLOTS] = [ROW_LOW, ROW_MID, ROW_HIGH];

/// Number of context actions; the array in [`layout`] must match. The grid
/// shapes itself around this, so it is not tied to a row count.
pub const CONTEXT_SLOTS: usize = 12;

/// The text on a button.
#[derive(Clone, Copy)]
enum Label {
    /// Fluent message id.
    Key(&'static str),
    /// A symbol that needs no translation.
    Symbol(&'static str),
    /// Two messages, picked by whether "More" is open.
    Toggle {
        closed: &'static str,
        open: &'static str,
    },
}

/// An always-visible button.
#[derive(Clone, Copy)]
struct Fixed {
    /// `None` for purely visual buttons (the movement stick).
    action: Option<Kind>,
    label: Label,
    lane: f32,
    row: f32,
    diameter: f32,
}

/// Always-visible buttons, in rows from the top of the screen down.
const FIXED: [Fixed; 14] = [
    // Menu row. Settings is left out on purpose: the menu button opens the
    // escape menu, which has it.
    Fixed {
        action: Some(Kind::Input(GameInput::Inventory)),
        label: Label::Key("hud-touch-inventory"),
        lane: MENU_LANES[0],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Diary)),
        label: Label::Key("hud-touch-diary"),
        lane: MENU_LANES[1],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Escape)),
        label: Label::Key("hud-touch-menu"),
        lane: MENU_LANES[2],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    // Skill slots.
    Fixed {
        action: Some(Kind::Input(GameInput::Slot1)),
        label: Label::Symbol("1"),
        lane: SLOT_LANES[0],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot2)),
        label: Label::Symbol("2"),
        lane: SLOT_LANES[1],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot3)),
        label: Label::Symbol("3"),
        lane: SLOT_LANES[2],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot4)),
        label: Label::Symbol("4"),
        lane: SLOT_LANES[3],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot5)),
        label: Label::Symbol("5"),
        lane: SLOT_LANES[4],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    // "More", above the context column so it never moves when the column fills.
    Fixed {
        action: Some(Kind::More),
        label: Label::Toggle {
            closed: "hud-touch-more",
            open: "hud-touch-less",
        },
        lane: LANE_MORE,
        row: ROW_SLOT,
        diameter: D_SMALL,
    },
    // Combat cluster: the attack button on the edge lane, the rest around it.
    Fixed {
        action: Some(Kind::Input(GameInput::Interact)),
        label: Label::Key("hud-touch-interact"),
        lane: LANE_EDGE,
        row: ROW_HIGH,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Jump)),
        label: Label::Key("hud-touch-jump"),
        lane: LANE_EDGE,
        row: ROW_MID,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Secondary)),
        label: Label::Key("hud-touch-alt"),
        lane: LANE_INNER,
        row: ROW_MID,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Roll)),
        label: Label::Key("hud-touch-roll"),
        lane: LANE_INNER,
        row: ROW_LOW,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Primary)),
        label: Label::Key("hud-touch-attack"),
        lane: LANE_EDGE,
        row: ROW_LOW,
        diameter: D_ATTACK,
    },
];

/// Total number of button slots; used to size the widget ids.
pub const COUNT: usize = 1 + FIXED.len() + CONTEXT_SLOTS;

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

/// Converts a lane and a row into fractions of the screen.
fn center_of(lane: f32, row: f32, aspect: f32) -> Vec2<f32> {
    // A zero aspect would only happen before the window has a size.
    Vec2::new(1.0 - lane / aspect.max(0.1), row)
}

/// The context grid as it fits on one screen, in screen-height units.
#[derive(Clone, Copy, Debug)]
struct Grid {
    /// Centre of the grid, measured from the left edge.
    center: f32,
    columns: usize,
    /// Distance between neighbouring column centres.
    pitch: f32,
    /// Distance between neighbouring row centres.
    row_pitch: f32,
    /// Button diameter, which shrinks on narrow screens so that nothing
    /// touches.
    diameter: f32,
}

/// Measures the context grid out of the space between the movement stick and
/// the combat cluster.
///
/// The grid takes as many columns as fit at the usual size and adds rows
/// instead when the screen is too narrow for them, shrinking its buttons only
/// as far as it has to. From a 4:3 tablet upwards that leaves the plain 3x4
/// grid; on a squarer window it becomes 2x6 with slightly smaller buttons
/// rather than a pile of touching ones.
fn grid(aspect: f32) -> Grid {
    let stick = STICK_X * aspect + D_STICK / 2.0;
    let cluster = aspect - LANE_INNER - D_SMALL / 2.0;
    let free = (cluster - stick - 2.0 * PANEL_MARGIN).max(0.0);
    let columns = ((free + BUTTON_GAP) / (D_SMALL + BUTTON_GAP))
        .floor()
        .clamp(PANEL_COLS_MIN as f32, PANEL_COLS as f32) as usize;
    let rows = CONTEXT_SLOTS.div_ceil(columns);
    let row_pitch = (PANEL_ROW_BOTTOM - PANEL_ROW_TOP) / (rows - 1).max(1) as f32;
    let row_pitch = row_pitch.min(PANEL_ROW_PITCH);
    // Buttons give up size before they give up the gap, across and down.
    let diameter = D_SMALL
        .min((free - (columns - 1) as f32 * BUTTON_GAP) / columns as f32)
        .min(row_pitch - BUTTON_GAP)
        .max(PANEL_D_MIN);
    let spread = (free - diameter) / (columns - 1) as f32;
    Grid {
        center: (stick + cluster) / 2.0,
        columns,
        pitch: spread.clamp(diameter + BUTTON_GAP, PANEL_SPACING_MAX),
        row_pitch,
        diameter,
    }
}

/// Where one button goes: a lane, a row, and how big it is.
struct Placement {
    lane: f32,
    row: f32,
    diameter: f32,
}

/// Lane and row of a context action, or `None` if it is not offered right now.
fn context_slot(slot: usize, expanded: bool, aspect: f32) -> Option<Placement> {
    if !expanded {
        return Some(Placement {
            lane: LANE_CONTEXT,
            row: *TIER_ROWS.get(slot)?,
            diameter: D_SMALL,
        });
    }
    let grid = grid(aspect);
    let column = slot % grid.columns;
    let row = slot / grid.columns;
    // The grid is measured from the left, lanes from the right.
    let from_left = grid.center + (column as f32 - (grid.columns - 1) as f32 / 2.0) * grid.pitch;
    Some(Placement {
        lane: aspect - from_left,
        row: PANEL_ROW_TOP + row as f32 * grid.row_pitch,
        diameter: grid.diameter,
    })
}

/// Returns the buttons to show for the current state.
///
/// `expanded` is the state of the "More" button and `aspect` is the window
/// width divided by its height.
pub fn shown(
    ctx: &Context,
    expanded: bool,
    aspect: f32,
    localize: impl Fn(&str) -> String,
) -> Vec<Shown> {
    let mut shown = Vec::with_capacity(COUNT);
    shown.push(Shown {
        index: 0,
        action: None,
        center: Vec2::new(STICK_X, STICK_Y),
        diameter: D_STICK,
        label: localize("hud-touch-move"),
        active: false,
    });
    for (offset, button) in FIXED.iter().enumerate() {
        shown.push(Shown {
            index: offset + 1,
            action: button.action,
            center: center_of(button.lane, button.row, aspect),
            diameter: button.diameter,
            label: match button.label {
                Label::Key(key) => localize(key),
                Label::Symbol(symbol) => symbol.to_owned(),
                Label::Toggle { closed, open } => localize(if expanded { open } else { closed }),
            },
            active: matches!(button.action, Some(Kind::More)) && expanded,
        });
    }
    for spec in layout(ctx) {
        if !spec.enabled {
            continue;
        }
        let Some(placement) = context_slot(spec.slot, expanded, aspect) else {
            continue;
        };
        shown.push(Shown {
            index: 1 + FIXED.len() + spec.slot,
            action: Some(spec.kind),
            center: center_of(placement.lane, placement.row, aspect),
            diameter: placement.diameter,
            label: localize(spec.label_key),
            active: spec.active,
        });
    }
    shown
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shape the overlay has to work on. The Android activity is locked
    /// to landscape, so a square multi-window pane is the narrowest the overlay
    /// can be given; 21:9 is the widest phone in common use, and 3:1 stands in
    /// for a window dragged wide on a desktop.
    const ASPECTS: [f32; 10] = [
        1.0,
        1.1,
        1.2,
        4.0 / 3.0,
        3.0 / 2.0,
        16.0 / 9.0,
        2.0,
        19.5 / 9.0,
        21.0 / 9.0,
        3.0,
    ];

    /// How close two touch targets may sit, in screen-height units. Anything
    /// under this is a thumb trap even where the circles do not quite touch.
    /// The layout keeps [`PANEL_MARGIN`] — a little more — at its tightest; the
    /// difference is slack for the rounding in these positions.
    const MIN_GAP: f32 = 0.015;

    /// A state where every action is available, so the layout is at its
    /// fullest.
    fn full_context() -> Context {
        Context {
            controlling: true,
            riding: false,
            wielding: true,
            gliding: false,
            has_glider: true,
            has_lantern: true,
            lantern_on: true,
            sneaking: true,
            sitting: true,
            crawling: true,
            dancing: true,
            zoom_locked: false,
        }
    }

    /// Every button on screen, in screen-height units: `(index, x, y, radius)`.
    fn placed(aspect: f32, expanded: bool) -> Vec<(usize, f32, f32, f32)> {
        shown(&full_context(), expanded, aspect, str::to_owned)
            .into_iter()
            .map(|b| (b.index, b.center.x * aspect, b.center.y, b.diameter / 2.0))
            .collect()
    }

    #[test]
    fn buttons_do_not_overlap() {
        for aspect in ASPECTS {
            for expanded in [false, true] {
                let buttons = placed(aspect, expanded);
                for (i, &(index, x, y, radius)) in buttons.iter().enumerate() {
                    for &(other, x2, y2, radius2) in &buttons[i + 1..] {
                        let gap = ((x - x2).powi(2) + (y - y2).powi(2)).sqrt() - radius - radius2;
                        assert!(
                            gap >= MIN_GAP,
                            "slots {index}/{other} gap {gap:.4}h at {aspect:.2} open={expanded}",
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn buttons_stay_on_screen() {
        for aspect in ASPECTS {
            for expanded in [false, true] {
                for (index, x, y, radius) in placed(aspect, expanded) {
                    assert!(
                        x - radius >= 0.0 && x + radius <= aspect,
                        "slot {index} leaves the screen sideways at aspect {aspect:.2}",
                    );
                    assert!(
                        y - radius >= 0.0 && y + radius <= 1.0,
                        "slot {index} leaves the screen vertically at aspect {aspect:.2}",
                    );
                }
            }
        }
    }

    #[test]
    fn every_slot_has_its_own_widget_id() {
        let buttons = shown(&full_context(), true, 16.0 / 9.0, str::to_owned);
        let mut indexes: Vec<usize> = buttons.iter().map(|button| button.index).collect();
        indexes.sort_unstable();
        indexes.dedup();
        assert_eq!(indexes.len(), buttons.len(), "two buttons share a slot");
        assert!(
            indexes.iter().all(|&index| index < COUNT),
            "a button has no widget id",
        );
    }

    #[test]
    fn opening_more_offers_every_action() {
        let closed = shown(&full_context(), false, 16.0 / 9.0, str::to_owned);
        let open = shown(&full_context(), true, 16.0 / 9.0, str::to_owned);
        let contexts = |buttons: &[Shown]| {
            buttons
                .iter()
                .filter(|button| button.index > FIXED.len())
                .count()
        };
        assert_eq!(contexts(&closed), TIER_SLOTS);
        assert_eq!(contexts(&open), CONTEXT_SLOTS);
    }
}
