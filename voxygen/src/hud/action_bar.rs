//! Context-sensitive on-screen action bar.
//!
//! Actions that are otherwise only reachable through key bindings (gliding,
//! blocking, sheathing weapons, toggling the lantern, zooming, ...) are also
//! offered as buttons. Every entry is drawn by the same reusable [`Row`]
//! widget, and [`layout`] decides which entries are relevant for the player's
//! current state, so the bar stays short. Less common actions sit behind a
//! "More" button that expands a second row.
//!
//! Pressing a button forwards the same [`GameInput`] that the matching key
//! would send, so all gameplay logic remains in the session.

use super::TEXT_COLOR;
use crate::game_input::GameInput;
use conrod_core::{
    Color, Colorable, Labelable, Positionable, Sizeable, UiCell, Widget,
    text::font,
    widget::{self, Button, Rectangle},
};
use hashbrown::HashSet;

/// Maximum number of buttons in one row. Sizes the widget id pool.
pub const MAX_ROW_LEN: usize = 12;
/// Number of rows: the primary row and the expandable secondary row.
pub const ROWS: usize = 2;
/// Distance from the bottom of the window to the primary row.
pub const BOTTOM_MARGIN: f64 = 150.0;
/// Vertical distance between two stacked rows.
pub const ROW_STEP: f64 = BUTTON_H + 2.0 * PAD + 8.0;

const BUTTON_W: f64 = 84.0;
const BUTTON_H: f64 = 42.0;
const GAP: f64 = 6.0;
const PAD: f64 = 6.0;

const INACTIVE: Color = Color::Rgba(0.10, 0.13, 0.18, 0.92);
const ACTIVE: Color = Color::Rgba(0.85, 0.62, 0.18, 0.95);
const DISABLED: Color = Color::Rgba(0.07, 0.08, 0.10, 0.55);
const HOVER: Color = Color::Rgba(0.22, 0.30, 0.40, 0.97);
const PRESS: Color = Color::Rgba(0.32, 0.42, 0.56, 1.0);
const DISABLED_TEXT: Color = Color::Rgba(0.45, 0.45, 0.45, 0.8);

/// What a button does when pressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Sends the input exactly like pressing its key.
    Input(GameInput),
    /// Expands or collapses the secondary row.
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
    /// Shows the button in its "on" colour, e.g. while gliding.
    pub active: bool,
    /// Disabled buttons are drawn dimmed and do not send input.
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

/// A [`Spec`] with its label already resolved, ready to draw.
#[derive(Clone, Debug)]
pub struct Item {
    pub spec: Spec,
    pub label: String,
}

/// Returns the primary row and the secondary row for the given state.
///
/// The secondary row is empty unless `expanded` is set.
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

/// Resolves labels for `specs`. Input buttons show their bound key on a second
/// line when one is set.
pub fn items(
    specs: &[Spec],
    localize: impl Fn(&str) -> String,
    key_hint: impl Fn(GameInput) -> Option<String>,
) -> Vec<Item> {
    specs
        .iter()
        .take(MAX_ROW_LEN)
        .map(|spec| {
            let name = localize(spec.label_key);
            let label = match spec.kind {
                Kind::Input(input) => match key_hint(input) {
                    Some(key) => format!("{name}\n[{key}]"),
                    None => name,
                },
                Kind::More => name,
            };
            Item { spec: *spec, label }
        })
        .collect()
}

/// Result of drawing one row for this frame.
#[derive(Default)]
pub struct Outcome {
    /// Inputs whose buttons are currently held down with the mouse.
    pub held: HashSet<GameInput>,
    /// The "More" button was clicked this frame.
    pub more_clicked: bool,
}

/// One row of the bar. Buttons are laid out left to right and centred at the
/// bottom of the window.
pub struct Row<'a> {
    pub items: &'a [Item],
    /// Widget ids for the buttons, at least `items.len()` of them.
    pub ids: &'a [widget::Id],
    pub background: widget::Id,
    pub window: widget::Id,
    pub bottom_margin: f64,
    pub font: font::Id,
    pub font_size: u32,
}

impl Row<'_> {
    pub fn draw(self, ui: &mut UiCell) -> Outcome {
        let mut outcome = Outcome::default();
        let count = self.items.len().min(self.ids.len());
        if count == 0 {
            return outcome;
        }

        let width = count as f64 * BUTTON_W + (count - 1) as f64 * GAP + 2.0 * PAD;
        let height = BUTTON_H + 2.0 * PAD;
        Rectangle::fill([width, height])
            .rgba(0.03, 0.05, 0.08, 0.55)
            .mid_bottom_with_margin_on(self.window, self.bottom_margin)
            .set(self.background, ui);

        for (i, item) in self.items.iter().take(count).enumerate() {
            let id = self.ids[i];
            let spec = item.spec;

            let (base, text) = match (spec.enabled, spec.active) {
                (false, _) => (DISABLED, DISABLED_TEXT),
                (true, true) => (ACTIVE, TEXT_COLOR),
                (true, false) => (INACTIVE, TEXT_COLOR),
            };
            let button = Button::new()
                .w_h(BUTTON_W, BUTTON_H)
                .color(base)
                .hover_color(HOVER)
                .press_color(PRESS)
                .label(&item.label)
                .label_color(text)
                .label_font_size(self.font_size)
                .label_font_id(self.font);
            let button = if i == 0 {
                button.mid_left_with_margin_on(self.background, PAD)
            } else {
                button.right_from(self.ids[i - 1], GAP)
            };
            let clicked = button.set(id, ui).was_clicked();

            match spec.kind {
                Kind::Input(input) => {
                    // Input buttons act on press and release, so the widget
                    // reports whether the mouse is still down on it. Holding
                    // Block works this way. Toggles just see press and release.
                    let held = spec.enabled
                        && ui
                            .widget_input(id)
                            .mouse()
                            .is_some_and(|mouse| mouse.buttons.left().is_down());
                    if held {
                        outcome.held.insert(input);
                    }
                },
                Kind::More => outcome.more_clicked |= clicked,
            }
        }

        outcome
    }
}
