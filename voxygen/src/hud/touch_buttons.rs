//! On-screen touch buttons for phones (Android), drawn over the game while the
//! pointer is grabbed.
//!
//! Every button is described by the same data: the HUD draws it and the session
//! hit-tests the same region, so the two cannot drift apart.
//!
//! # Layout
//!
//! Diameters and rows are fractions of the screen *height*. Left and right
//! lanes are measured inwards from their respective edges, so one unit means
//! the same physical size on a 4:3 tablet and on a 21:9 phone. Callers
//! pass the aspect ratio; [`Shown::center`] comes back as fractions of the
//! screen, which is what the drawing code and the hit-testing already speak.
//!
//! The screen is organised in rows (menus, skill slots, then the combat
//! cluster) and lanes (left, edge, inner, context). Menu and skill-slot lanes
//! stay to the left of the default top-right minimap; combat lanes are counted
//! in from the right. Two rules keep it from getting crowded:
//!
//! - The context actions form a single list. While "More" is closed, only the
//!   first [`TIER_SLOTS`] enabled actions are offered in a column beside the
//!   combat cluster. Opening "More" swaps that column for a grid with room for
//!   every context action, so the screen never carries two piles of them at once.
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
    /// A mountable entity or block is in interaction range.
    pub has_mount_target: bool,
    /// A tradeable entity is in interaction range.
    pub has_trade_target: bool,
    /// An owned pet can be told to stay or follow.
    pub has_stay_follow_target: bool,
    /// Whether the target pet is currently staying.
    pub pet_staying: bool,
    /// A trade is active; Glide and Sneak inputs are ignored then.
    pub trading: bool,
    pub wielding: bool,
    pub gliding: bool,
    /// A glider is equipped, so gliding can start.
    pub has_glider: bool,
    /// A lantern is equipped, so the lantern can be turned on.
    pub has_lantern: bool,
    pub lantern_on: bool,
    /// The scene around the player is dark enough to benefit from a lantern.
    pub is_dark: bool,
    /// An Interact-key target or active dialogue is currently available.
    pub has_interactable: bool,
    pub sneaking: bool,
    pub sitting: bool,
    pub crawling: bool,
    pub dancing: bool,
    pub zoom_locked: bool,
    /// The character has died; keep only menus and the respawn control visible.
    pub dead: bool,
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
    /// Stable place in the context list and expanded grid. The closed
    /// layout compacts enabled actions into its first [`TIER_SLOTS`] positions.
    pub slot: usize,
}

/// The context actions, in slot order. Nearby interactions come first so they
/// are visible without opening "More"; then come combat, toggles and emotes,
/// and finally the camera.
///
/// Whether an action is offered at all comes from [`state_of`], which keeps
/// this a plain list.
const ACTIONS: [(GameInput, &str); CONTEXT_SLOTS] = [
    (GameInput::Mount, "hud-mount"),
    (GameInput::Trade, "hud-trade"),
    (GameInput::StayFollow, "hud-stay"),
    (GameInput::ToggleWield, "hud-touch-draw"),
    (GameInput::Block, "hud-touch-block"),
    (GameInput::ToggleLantern, "hud-touch-lantern"),
    (GameInput::Glide, "hud-touch-glide"),
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
            // Some actions use one input to switch between two states.
            let label_key = match input {
                GameInput::Mount if ctx.riding => "hud-unmount",
                GameInput::StayFollow if ctx.pet_staying => "hud-follow",
                GameInput::StayFollow => "hud-stay",
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
    // Gameplay actions are unavailable while dead; menus stay visible and
    // respawn has its own dedicated button.
    let ready = ctx.controlling && !ctx.riding && !ctx.dead;
    let still = ready && !ctx.gliding;
    let glider = ctx.gliding || ctx.has_glider;
    let lantern = ctx.has_lantern || ctx.lantern_on;
    match input {
        GameInput::Mount => (
            ctx.riding,
            ctx.controlling && !ctx.dead && (ctx.riding || ctx.has_mount_target),
        ),
        GameInput::Trade => (
            false,
            ctx.controlling && !ctx.dead && !ctx.trading && ctx.has_trade_target,
        ),
        GameInput::StayFollow => (
            ctx.pet_staying,
            ctx.controlling && !ctx.dead && ctx.has_stay_follow_target,
        ),
        GameInput::ToggleWield => (ctx.wielding, ready),
        GameInput::Block => (false, still && ctx.wielding),
        GameInput::Glide => (ctx.gliding, ready && glider && !ctx.trading),
        // Leave the toggle available while it is on so the player can turn it
        // back off if the surroundings brighten.
        GameInput::ToggleLantern => (
            ctx.lantern_on,
            ready && lantern && (ctx.is_dark || ctx.lantern_on),
        ),
        GameInput::Sneak => (ctx.sneaking, still && !ctx.trading),
        GameInput::Sit => (ctx.sitting, still),
        GameInput::Crawl => (ctx.crawling, still),
        GameInput::Dance => (ctx.dancing, still),
        GameInput::Greet => (false, still),
        // Zooming does nothing while the zoom is locked, so hide it then.
        GameInput::ZoomIn | GameInput::ZoomOut => (false, ready && !ctx.zoom_locked),
        GameInput::ZoomLock => (ctx.zoom_locked, ready),
        _ => (false, false),
    }
}

/// Diameters, as fractions of the screen height.
const D_STICK: f32 = 0.30;
const D_ATTACK: f32 = 0.18;
const D_SMALL: f32 = 0.13;
const D_SLOT: f32 = 0.089;
const D_MENU: f32 = 0.085;

/// Rows, as fractions of the screen height measured from the top.
const ROW_MENU: f32 = 0.055;
const ROW_SLOT: f32 = 0.175;
const ROW_HIGH: f32 = 0.395;
const ROW_MID: f32 = 0.585;
const ROW_LOW: f32 = 0.80;

/// Right-hand lanes, in screen-height units measured inwards from the edge.
const LANE_EDGE: f32 = 0.13;
const LANE_INNER: f32 = 0.345;
/// Where the context column sits while "More" is closed.
const LANE_CONTEXT: f32 = 0.56;

/// Left-hand lanes of the five skill slots and three menu buttons. The top row
/// is deliberately kept clear of the minimap.
const SLOT_LANES: [f32; 5] = [0.20, 0.305, 0.41, 0.515, 0.62];
const MENU_LANES: [f32; 3] = [0.20, 0.31, 0.42];

/// The movement stick is placed by width instead, so it stays under the left
/// thumb however wide the screen is.
const STICK_X: f32 = 0.16;
const STICK_Y: f32 = 0.73;

/// Most columns the context grid uses, on a screen wide enough for them.
const PANEL_COLS: usize = 3;
/// Fewest columns it falls back to. A single column of fifteen would not fit
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
pub const CONTEXT_SLOTS: usize = 15;

/// Buttons that exist in one state only. They come after the context actions so
/// that every widget id stays where it is.
const DEATH_SLOTS: usize = 1;
/// Row of the respawn button, and where it sits across the screen. It belongs
/// to neither cluster, so it is placed by width instead of by lane.
const ROW_RESPAWN: f32 = 0.88;
const X_RESPAWN: f32 = 0.5;

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

/// Which edge a fixed button's lane is measured from.
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

/// A button with a fixed position in the layout.
#[derive(Clone, Copy)]
struct Fixed {
    /// `None` for purely visual buttons (the movement stick).
    action: Option<Kind>,
    label: Label,
    side: Side,
    lane: f32,
    row: f32,
    diameter: f32,
}

/// Fixed-position buttons, in rows from the top of the screen down.
const FIXED: [Fixed; 14] = [
    // Menu row. Settings is left out on purpose: the menu button opens the
    // escape menu, which has it.
    Fixed {
        action: Some(Kind::Input(GameInput::Inventory)),
        label: Label::Key("hud-touch-inventory"),
        side: Side::Left,
        lane: MENU_LANES[0],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Diary)),
        label: Label::Key("hud-touch-diary"),
        side: Side::Left,
        lane: MENU_LANES[1],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Escape)),
        label: Label::Key("hud-touch-menu"),
        side: Side::Left,
        lane: MENU_LANES[2],
        row: ROW_MENU,
        diameter: D_MENU,
    },
    // Skill slots.
    Fixed {
        action: Some(Kind::Input(GameInput::Slot1)),
        label: Label::Symbol("1"),
        side: Side::Left,
        lane: SLOT_LANES[0],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot2)),
        label: Label::Symbol("2"),
        side: Side::Left,
        lane: SLOT_LANES[1],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot3)),
        label: Label::Symbol("3"),
        side: Side::Left,
        lane: SLOT_LANES[2],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot4)),
        label: Label::Symbol("4"),
        side: Side::Left,
        lane: SLOT_LANES[3],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Slot5)),
        label: Label::Symbol("5"),
        side: Side::Left,
        lane: SLOT_LANES[4],
        row: ROW_SLOT,
        diameter: D_SLOT,
    },
    // "More", at the left end of the menu row, clear of the minimap.
    Fixed {
        action: Some(Kind::More),
        label: Label::Toggle {
            closed: "hud-touch-more",
            open: "hud-touch-less",
        },
        side: Side::Left,
        lane: 0.58,
        row: ROW_MENU,
        diameter: D_MENU,
    },
    // Combat cluster: the attack button on the edge lane, the rest around it.
    Fixed {
        action: Some(Kind::Input(GameInput::Interact)),
        label: Label::Key("hud-touch-interact"),
        side: Side::Right,
        lane: LANE_EDGE,
        row: ROW_HIGH,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Jump)),
        label: Label::Key("hud-touch-jump"),
        side: Side::Right,
        lane: LANE_EDGE,
        row: ROW_MID,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Secondary)),
        label: Label::Key("hud-touch-alt"),
        side: Side::Right,
        lane: LANE_INNER,
        row: ROW_MID,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Roll)),
        label: Label::Key("hud-touch-roll"),
        side: Side::Right,
        lane: LANE_INNER,
        row: ROW_LOW,
        diameter: D_SMALL,
    },
    Fixed {
        action: Some(Kind::Input(GameInput::Primary)),
        label: Label::Key("hud-touch-attack"),
        side: Side::Right,
        lane: LANE_EDGE,
        row: ROW_LOW,
        diameter: D_ATTACK,
    },
];

/// Total number of button slots; used to size the widget ids.
pub const COUNT: usize = 1 + FIXED.len() + CONTEXT_SLOTS + DEATH_SLOTS;

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
fn center_of(side: Side, lane: f32, row: f32, aspect: f32) -> Vec2<f32> {
    // A zero aspect would only happen before the window has a size.
    let aspect = aspect.max(0.1);
    let x = match side {
        Side::Left => lane / aspect,
        Side::Right => 1.0 - lane / aspect,
    };
    Vec2::new(x, row)
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
/// as far as it has to. From a 4:3 tablet upwards that leaves the plain 3x5
/// grid; on a squarer window it becomes 2x8 with slightly smaller buttons
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
    if !ctx.dead {
        shown.push(Shown {
            index: 0,
            action: None,
            center: Vec2::new(STICK_X, STICK_Y),
            diameter: D_STICK,
            label: localize("hud-touch-move"),
            active: false,
        });
    }
    for (offset, button) in FIXED.iter().enumerate() {
        // Death leaves menu access and the dedicated respawn control; hide
        // movement, hotbar, context and combat controls.
        if ctx.dead
            && !matches!(
                button.action,
                Some(Kind::Input(GameInput::Inventory | GameInput::Diary | GameInput::Escape))
            )
        {
            continue;
        }
        // Interact is useful only while a live character has something in
        // range; keep its widget id stable when it is hidden.
        if matches!(button.action, Some(Kind::Input(GameInput::Interact)))
            && (!ctx.controlling || ctx.dead || !ctx.has_interactable)
        {
            continue;
        }
        shown.push(Shown {
            index: offset + 1,
            action: button.action,
            center: center_of(button.side, button.lane, button.row, aspect),
            diameter: button.diameter,
            label: match button.label {
                Label::Key(key) => localize(key),
                Label::Symbol(symbol) => symbol.to_owned(),
                Label::Toggle { closed, open } => localize(if expanded { open } else { closed }),
            },
            active: matches!(button.action, Some(Kind::More)) && expanded,
        });
    }
    let mut tier_slot = 0;
    for spec in layout(ctx) {
        if !spec.enabled {
            continue;
        }
        let display_slot = if expanded {
            spec.slot
        } else {
            if tier_slot >= TIER_SLOTS {
                continue;
            }
            let slot = tier_slot;
            tier_slot += 1;
            slot
        };
        let Some(placement) = context_slot(display_slot, expanded, aspect) else {
            continue;
        };
        shown.push(Shown {
            index: 1 + FIXED.len() + spec.slot,
            action: Some(spec.kind),
            center: center_of(Side::Right, placement.lane, placement.row, aspect),
            diameter: placement.diameter,
            label: localize(spec.label_key),
            active: spec.active,
        });
    }
    if ctx.dead {
        // Dying leaves nothing to do but respawn, and on a phone there is no
        // key to press for it, so it gets a button of its own.
        shown.push(Shown {
            index: 1 + FIXED.len() + CONTEXT_SLOTS,
            action: Some(Kind::Input(GameInput::Respawn)),
            center: Vec2::new(X_RESPAWN, ROW_RESPAWN),
            diameter: D_ATTACK,
            label: localize("hud-touch-respawn"),
            active: true,
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
            has_mount_target: true,
            has_trade_target: true,
            has_stay_follow_target: true,
            pet_staying: false,
            trading: false,
            wielding: true,
            gliding: false,
            has_glider: true,
            has_lantern: true,
            lantern_on: true,
            is_dark: true,
            has_interactable: true,
            sneaking: true,
            sitting: true,
            crawling: true,
            dancing: true,
            zoom_locked: false,
            dead: false,
        }
    }

    /// The state the player is in after dying: only menus and respawn remain.
    fn dead_context() -> Context {
        Context {
            dead: true,
            ..full_context()
        }
    }

    /// Every state the overlay is drawn in.
    fn contexts() -> [(&'static str, Context); 2] {
        [("alive", full_context()), ("dead", dead_context())]
    }

    /// Every button on screen, in screen-height units: `(index, x, y, radius)`.
    fn placed(ctx: &Context, aspect: f32, expanded: bool) -> Vec<(usize, f32, f32, f32)> {
        shown(ctx, expanded, aspect, str::to_owned)
            .into_iter()
            .map(|b| (b.index, b.center.x * aspect, b.center.y, b.diameter / 2.0))
            .collect()
    }

    /// Fails if any two of these buttons come within [`MIN_GAP`] of each other.
    fn assert_apart(buttons: &[(usize, f32, f32, f32)], state: &str) {
        for (i, &(index, x, y, radius)) in buttons.iter().enumerate() {
            for &(other, x2, y2, radius2) in &buttons[i + 1..] {
                let gap = ((x - x2).powi(2) + (y - y2).powi(2)).sqrt() - radius - radius2;
                assert!(
                    gap >= MIN_GAP,
                    "slots {index}/{other} gap {gap:.4}h in {state}",
                );
            }
        }
    }

    fn has_input(buttons: &[Shown], input: GameInput) -> bool {
        buttons.iter().any(|button| {
            matches!(button.action, Some(Kind::Input(action)) if action == input)
        })
    }

    #[test]
    fn lantern_is_offered_only_when_dark_or_already_on() {
        let mut ctx = full_context();
        ctx.is_dark = false;
        ctx.lantern_on = false;
        let bright = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(!has_input(&bright, GameInput::ToggleLantern));

        ctx.is_dark = true;
        let dark = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(has_input(&dark, GameInput::ToggleLantern));

        ctx.is_dark = false;
        ctx.lantern_on = true;
        let already_on = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(has_input(&already_on, GameInput::ToggleLantern));

        ctx.has_lantern = false;
        ctx.lantern_on = false;
        ctx.is_dark = true;
        let no_lantern = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(!has_input(&no_lantern, GameInput::ToggleLantern));
    }

    #[test]
    fn interact_is_offered_only_for_a_live_player_with_a_target() {
        let mut ctx = full_context();
        ctx.has_interactable = false;
        let no_target = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(!has_input(&no_target, GameInput::Interact));

        ctx.has_interactable = true;
        let in_range = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(has_input(&in_range, GameInput::Interact));

        ctx.dead = true;
        let dead = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        assert!(!has_input(&dead, GameInput::Interact));
    }

    #[test]
    fn nearby_mount_trade_and_pet_actions_are_dynamic_and_tappable() {
        let mut ctx = full_context();
        ctx.has_mount_target = false;
        ctx.has_trade_target = false;
        ctx.has_stay_follow_target = false;
        let no_targets = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        for input in [GameInput::Mount, GameInput::Trade, GameInput::StayFollow] {
            assert!(!has_input(&no_targets, input), "unexpected {input:?} button");
        }

        ctx.has_mount_target = true;
        ctx.has_trade_target = true;
        ctx.has_stay_follow_target = true;
        let nearby = shown(&ctx, false, 16.0 / 9.0, str::to_owned);
        let tap_regions = regions(&nearby);
        for input in [GameInput::Mount, GameInput::Trade, GameInput::StayFollow] {
            let button = nearby
                .iter()
                .find(|button| {
                    matches!(button.action, Some(Kind::Input(action)) if action == input)
                })
                .unwrap_or_else(|| panic!("missing {input:?} button"));
            let region = tap_regions
                .iter()
                .find(|region| matches!(region.action, Kind::Input(action) if action == input))
                .unwrap_or_else(|| panic!("{input:?} is drawn without a matching tap region"));
            assert_eq!(region.center, button.center);
            assert_eq!(region.radius, button.diameter / 2.0);
        }

        // Nearby interactions take the three always-visible context slots.
        let tier: Vec<_> = nearby
            .iter()
            .filter(|button| {
                button.index > FIXED.len() && button.index <= FIXED.len() + CONTEXT_SLOTS
            })
            .filter_map(|button| match button.action {
                Some(Kind::Input(input)) => Some(input),
                _ => None,
            })
            .collect();
        assert_eq!(
            tier,
            vec![GameInput::Mount, GameInput::Trade, GameInput::StayFollow],
            "nearby interactions should not be hidden behind More",
        );

        // Mount remains available as a dismount action while already riding.
        ctx.riding = true;
        ctx.has_mount_target = false;
        let riding = layout(&ctx);
        let mount = riding
            .iter()
            .find(|spec| spec.kind == Kind::Input(GameInput::Mount))
            .expect("mount slot");
        assert!(mount.enabled && mount.active);
        assert_eq!(mount.label_key, "hud-unmount");

        // The pet toggle's text describes the action it will perform.
        ctx.pet_staying = true;
        let staying = layout(&ctx);
        let stay_follow = staying
            .iter()
            .find(|spec| spec.kind == Kind::Input(GameInput::StayFollow))
            .expect("stay/follow slot");
        assert!(stay_follow.active);
        assert_eq!(stay_follow.label_key, "hud-follow");

        // The trade input is not accepted while a trade is already underway.
        // Keep the player on foot so the Glide/Sneak checks exercise the
        // trading gate rather than their separate riding gate.
        ctx.riding = false;
        ctx.trading = false;
        let not_trading = shown(&ctx, true, 16.0 / 9.0, str::to_owned);
        assert!(has_input(&not_trading, GameInput::Glide));
        assert!(has_input(&not_trading, GameInput::Sneak));

        ctx.trading = true;
        let trading = shown(&ctx, true, 16.0 / 9.0, str::to_owned);
        assert!(!has_input(&trading, GameInput::Trade));
        assert!(!has_input(&trading, GameInput::Glide));
        assert!(!has_input(&trading, GameInput::Sneak));
    }

    #[test]
    fn menu_and_hotbar_buttons_are_clear_of_the_top_right_minimap() {
        let buttons = shown(&full_context(), false, 1.0, str::to_owned);
        // At aspect 1, this leaves a small buffer before a top-right minimap at
        // its largest setting (scale 2.0).
        let clear_of_minimap = |input| {
            let button = buttons
                .iter()
                .find(|button| {
                    matches!(button.action, Some(Kind::Input(action)) if action == input)
                })
                .expect("expected fixed button");
            button.center.x + button.diameter / 2.0 <= 0.67
        };
        for input in [
            GameInput::Inventory,
            GameInput::Diary,
            GameInput::Escape,
            GameInput::Slot1,
            GameInput::Slot2,
            GameInput::Slot3,
            GameInput::Slot4,
            GameInput::Slot5,
        ] {
            assert!(clear_of_minimap(input), "{input:?} overlaps the minimap");
        }
        let more = buttons
            .iter()
            .find(|button| matches!(button.action, Some(Kind::More)))
            .expect("expected More button");
        assert!(more.center.x + more.diameter / 2.0 <= 0.67);
    }

    #[test]
    fn buttons_do_not_overlap() {
        for (name, ctx) in contexts() {
            for aspect in ASPECTS {
                for expanded in [false, true] {
                    let buttons = placed(&ctx, aspect, expanded);
                    let state = format!("{name} at {aspect:.2} open={expanded}");
                    assert_apart(&buttons, &state);
                }
            }
        }
    }

    #[test]
    fn buttons_stay_on_screen() {
        for (name, ctx) in contexts() {
            for aspect in ASPECTS {
                for expanded in [false, true] {
                    for (index, x, y, radius) in placed(&ctx, aspect, expanded) {
                        assert!(
                            x - radius >= 0.0 && x + radius <= aspect,
                            "slot {index} off screen sideways in {name} at {aspect:.2}",
                        );
                        assert!(
                            y - radius >= 0.0 && y + radius <= 1.0,
                            "slot {index} off screen vertically in {name} at {aspect:.2}",
                        );
                    }
                }
            }
        }
    }

    /// Dying on a phone has to leave a way back: there is no key to press.
    #[test]
    fn respawn_is_offered_only_when_dead() {
        let respawns = |ctx: &Context| {
            shown(ctx, false, 16.0 / 9.0, str::to_owned)
                .iter()
                .filter(|button| matches!(button.action, Some(Kind::Input(GameInput::Respawn))))
                .count()
        };
        assert_eq!(respawns(&full_context()), 0);
        assert_eq!(respawns(&dead_context()), 1);
        // And it is tappable, not just drawn.
        let drawn = shown(&dead_context(), false, 16.0 / 9.0, str::to_owned);
        let tappable = regions(&drawn)
            .iter()
            .any(|region| matches!(region.action, Kind::Input(GameInput::Respawn)));
        assert!(tappable, "the respawn button has no region to tap");
        // Keep the three menu buttons, but hide the movement stick, hotbar,
        // context actions and combat cluster while dead.
        assert_eq!(drawn.len(), 4, "dead HUD should only have menus and respawn");
        for input in [GameInput::Inventory, GameInput::Diary, GameInput::Escape] {
            assert!(has_input(&drawn, input), "dead HUD lost menu action {input:?}");
        }
        for input in [
            GameInput::Slot1,
            GameInput::Jump,
            GameInput::Primary,
            GameInput::Secondary,
            GameInput::Roll,
        ] {
            assert!(!has_input(&drawn, input), "dead HUD still offers {input:?}");
        }
        assert!(
            !drawn.iter().any(|button| matches!(button.action, Some(Kind::More))),
            "More should be hidden when no context actions are available",
        );
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
                .filter(|button| {
                    button.index > FIXED.len() && button.index <= FIXED.len() + CONTEXT_SLOTS
                })
                .count()
        };
        assert_eq!(contexts(&closed), TIER_SLOTS);
        assert_eq!(contexts(&open), CONTEXT_SLOTS);
    }
}
