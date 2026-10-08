use crate::game_input::GameInput;
use vek::Vec2;

/// The five buttons in the touch action strip are reused for each page. This
/// keeps the number of on-screen targets small while still exposing actions
/// that do not fit in the default combat layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchActionPage {
    #[default]
    Hotbar,
    Combat,
    Utility,
    Movement,
}

impl TouchActionPage {
    pub const PAGE_COUNT: u8 = 4;

    pub const fn next(self) -> Self {
        match self {
            Self::Hotbar => Self::Combat,
            Self::Combat => Self::Utility,
            Self::Utility => Self::Movement,
            Self::Movement => Self::Hotbar,
        }
    }

    pub const fn number(self) -> u8 {
        match self {
            Self::Hotbar => 1,
            Self::Combat => 2,
            Self::Utility => 3,
            Self::Movement => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchButtonAction {
    Input(GameInput),
    NextPage,
}

#[derive(Clone, Copy, Debug)]
pub enum TouchButtonLabel {
    /// Language-neutral label, used for hotbar digits.
    Static(&'static str),
    /// Resolve the label from the button's `GameInput` localization key.
    LocalizedInput,
    /// A language-neutral page arrow and page count.
    PageIndicator,
}

#[derive(Clone, Copy, Debug)]
pub struct TouchButton {
    pub action: TouchButtonAction,
    /// Horizontal and vertical center in fractions of the physical window.
    pub x: f32,
    pub y: f32,
    /// Button diameter as a fraction of the physical window height.
    pub diameter: f32,
    pub label: TouchButtonLabel,
}

impl TouchButton {
    const fn fixed(input: GameInput, x: f32, y: f32, diameter: f32) -> Self {
        Self {
            action: TouchButtonAction::Input(input),
            x,
            y,
            diameter,
            label: TouchButtonLabel::LocalizedInput,
        }
    }

    const fn action(input: GameInput, x: f32) -> Self {
        let label = match input {
            GameInput::Slot1 => TouchButtonLabel::Static("1"),
            GameInput::Slot2 => TouchButtonLabel::Static("2"),
            GameInput::Slot3 => TouchButtonLabel::Static("3"),
            GameInput::Slot4 => TouchButtonLabel::Static("4"),
            GameInput::Slot5 => TouchButtonLabel::Static("5"),
            _ => TouchButtonLabel::LocalizedInput,
        };
        Self {
            action: TouchButtonAction::Input(input),
            x,
            y: 0.26,
            diameter: 0.10,
            label,
        }
    }
}

const FIXED_BUTTONS: [TouchButton; 9] = [
    TouchButton::fixed(GameInput::Primary, 0.90, 0.78, 0.15),
    TouchButton::fixed(GameInput::Secondary, 0.76, 0.85, 0.15),
    TouchButton::fixed(GameInput::Jump, 0.76, 0.66, 0.15),
    TouchButton::fixed(GameInput::Interact, 0.90, 0.57, 0.15),
    TouchButton::fixed(GameInput::Roll, 0.76, 0.47, 0.15),
    TouchButton::fixed(GameInput::Inventory, 0.56, 0.075, 0.09),
    TouchButton::fixed(GameInput::Diary, 0.66, 0.075, 0.09),
    TouchButton::fixed(GameInput::Settings, 0.76, 0.075, 0.09),
    TouchButton::fixed(GameInput::Escape, 0.86, 0.075, 0.09),
];

const PAGE_BUTTON: TouchButton = TouchButton {
    action: TouchButtonAction::NextPage,
    x: 0.95,
    y: 0.075,
    diameter: 0.09,
    label: TouchButtonLabel::PageIndicator,
};

const ACTION_BUTTON_COUNT: usize = 5;
pub const BUTTON_COUNT: usize = FIXED_BUTTONS.len() + ACTION_BUTTON_COUNT + 1;

/// Center and diameter of the left-side virtual movement stick.
pub const MOVE_STICK_X: f32 = 0.20;
pub const MOVE_STICK_Y: f32 = 0.76;
pub const MOVE_STICK_DIAMETER: f32 = 0.34;

/// Action-button locations are shared with the HUD drawing code, keeping the
/// rendered controls and touch hit regions in sync.
pub fn visible_buttons(page: TouchActionPage) -> impl Iterator<Item = TouchButton> {
    FIXED_BUTTONS
        .into_iter()
        .chain(action_buttons(page))
        .chain([PAGE_BUTTON])
}

fn action_buttons(page: TouchActionPage) -> [TouchButton; ACTION_BUTTON_COUNT] {
    [
        TouchButton::action(
            match page {
                TouchActionPage::Hotbar => GameInput::Slot1,
                TouchActionPage::Combat => GameInput::Block,
                TouchActionPage::Utility => GameInput::ZoomIn,
                TouchActionPage::Movement => GameInput::Sneak,
            },
            0.50,
        ),
        TouchButton::action(
            match page {
                TouchActionPage::Hotbar => GameInput::Slot2,
                TouchActionPage::Combat => GameInput::Glide,
                TouchActionPage::Utility => GameInput::ZoomOut,
                TouchActionPage::Movement => GameInput::ToggleWalk,
            },
            0.60,
        ),
        TouchButton::action(
            match page {
                TouchActionPage::Hotbar => GameInput::Slot3,
                TouchActionPage::Combat => GameInput::ToggleWield,
                TouchActionPage::Utility => GameInput::ZoomLock,
                TouchActionPage::Movement => GameInput::Sit,
            },
            0.70,
        ),
        TouchButton::action(
            match page {
                TouchActionPage::Hotbar => GameInput::Slot4,
                TouchActionPage::Combat => GameInput::ToggleLantern,
                TouchActionPage::Utility => GameInput::Mount,
                TouchActionPage::Movement => GameInput::Crawl,
            },
            0.80,
        ),
        TouchButton::action(
            match page {
                TouchActionPage::Hotbar => GameInput::Slot5,
                TouchActionPage::Combat => GameInput::SwapLoadout,
                TouchActionPage::Utility => GameInput::AutoWalk,
                TouchActionPage::Movement => GameInput::Dance,
            },
            0.90,
        ),
    ]
}

/// Find the touch button at a physical window position. This is the same
/// layout that `Hud::update_layout` draws, so targets cannot drift away from
/// their labels when pages change.
pub fn button_at(
    position: Vec2<f32>,
    width: f32,
    height: f32,
    page: TouchActionPage,
) -> Option<TouchButtonAction> {
    visible_buttons(page).find_map(|button| {
        let center = Vec2::new(button.x * width, button.y * height);
        let radius = height * button.diameter * 0.5;
        ((position - center).magnitude_squared() <= radius * radius).then_some(button.action)
    })
}

/// The movement stick accepts touches within the broad lower-left region so
/// players do not need to hit the exact center of its visual circle.
pub fn is_movement_stick_zone(position: Vec2<f32>, width: f32, height: f32) -> bool {
    position.x < width * 0.42 && position.y > height * 0.32
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGES: [TouchActionPage; 4] = [
        TouchActionPage::Hotbar,
        TouchActionPage::Combat,
        TouchActionPage::Utility,
        TouchActionPage::Movement,
    ];

    #[test]
    fn action_pages_expose_mobile_actions_and_keep_the_hotbar() {
        let actions = PAGES
            .into_iter()
            .flat_map(action_buttons)
            .filter_map(|button| match button.action {
                TouchButtonAction::Input(input) => Some(input),
                TouchButtonAction::NextPage => None,
            })
            .collect::<std::collections::HashSet<_>>();

        for input in [
            GameInput::Slot1,
            GameInput::Slot2,
            GameInput::Slot3,
            GameInput::Slot4,
            GameInput::Slot5,
            GameInput::Block,
            GameInput::Glide,
            GameInput::ToggleWield,
            GameInput::ToggleLantern,
            GameInput::ZoomIn,
            GameInput::ZoomOut,
        ] {
            assert!(actions.contains(&input), "missing touch action {input:?}");
        }
    }

    #[test]
    fn visible_button_count_matches_the_shared_layout() {
        for page in PAGES {
            assert_eq!(visible_buttons(page).count(), BUTTON_COUNT);
        }
    }

    #[test]
    fn action_pages_reuse_the_same_five_positions() {
        let positions = |page| {
            action_buttons(page)
                .map(|button| (button.x, button.y, button.diameter))
        };
        let first = positions(TouchActionPage::Hotbar);
        for page in PAGES {
            assert_eq!(positions(page), first);
        }
    }

    #[test]
    fn hit_testing_matches_the_drawn_button_layout() {
        let page = TouchActionPage::Combat;
        let button = action_buttons(page)[1];
        let width = 1920.0;
        let height = 1080.0;
        let center = Vec2::new(button.x * width, button.y * height);

        assert_eq!(
            button_at(center, width, height, page),
            Some(button.action)
        );
        assert_eq!(
            button_at(
                Vec2::new(PAGE_BUTTON.x * width, PAGE_BUTTON.y * height),
                width,
                height,
                page,
            ),
            Some(TouchButtonAction::NextPage)
        );
    }

    #[test]
    fn page_numbers_match_the_cycle_order() {
        for (page, number) in PAGES.into_iter().zip(1..=TouchActionPage::PAGE_COUNT) {
            assert_eq!(page.number(), number);
        }
    }

    #[test]
    fn page_and_action_buttons_have_labels() {
        assert!(FIXED_BUTTONS
            .iter()
            .all(|button| matches!(button.label, TouchButtonLabel::LocalizedInput)));
        assert!(action_buttons(TouchActionPage::Hotbar)
            .iter()
            .all(|button| matches!(button.label, TouchButtonLabel::Static(_))));
        assert!(action_buttons(TouchActionPage::Combat)
            .iter()
            .all(|button| matches!(button.label, TouchButtonLabel::LocalizedInput)));
        assert!(matches!(
            visible_buttons(TouchActionPage::Hotbar).last().unwrap().label,
            TouchButtonLabel::PageIndicator
        ));
    }

    #[test]
    fn movement_stick_accepts_the_lower_left_region() {
        assert!(is_movement_stick_zone(Vec2::new(20.0, 900.0), 1000.0, 1000.0));
        assert!(!is_movement_stick_zone(Vec2::new(900.0, 900.0), 1000.0, 1000.0));
        assert!(!is_movement_stick_zone(Vec2::new(20.0, 100.0), 1000.0, 1000.0));
    }

    #[test]
    fn pages_cycle_back_to_the_hotbar() {
        let mut page = TouchActionPage::Hotbar;
        for _ in 0..TouchActionPage::PAGE_COUNT {
            page = page.next();
        }
        assert_eq!(page, TouchActionPage::Hotbar);
    }
}
