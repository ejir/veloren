//! A widget for selecting a single value along some linear range.
use conrod_core::{
    Color, Colorable, Positionable, Rect, Sizeable, Widget, WidgetCommon, builder_methods, image,
    input::touch::Phase,
    position::{Point, Range},
    utils,
    widget::{self, Image},
    widget_ids,
};
use num::{Float, Integer, Num, NumCast};

pub enum Discrete {}
pub enum Continuous {}

pub trait ValueFromPercent<T> {
    fn value_from_percent(percent: f32, min: T, max: T) -> T;
}

/// Linear value selection.
///
/// If the slider's width is greater than its height, it will automatically
/// become a horizontal slider, otherwise it will be a vertical slider.
///
/// Its reaction is triggered if the value is updated or if the mouse button is
/// released while the cursor is above the rectangle.
#[derive(WidgetCommon)]
pub struct ImageSlider<T, K> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    // If `value > soft_max` we will display the slider at `soft_max` along with a faded ghost
    // slider at `value`. The slider displayed at `soft_max` is purely a visual indicator and has
    // no effect on the values produced by this slider.
    soft_max: T,
    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weigh lower values.
    ///
    /// Lower skew amounts (below 1.0) will weigh higher values.
    ///
    /// All skew amounts should be greater than 0.0.
    skew: f32,
    track: Track,
    slider: Slider,
    kind: std::marker::PhantomData<K>,
}

struct Track {
    image_id: image::Id,
    color: Option<Color>,
    // TODO: this is being set by users but we don't use it for anything here, figure out what it
    // is supposed to be used for
    breadth: Option<f32>,
    // Padding on the ends of the track constraining the slider to a smaller area.
    padding: (f32, f32),
}

struct Slider {
    image_id: image::Id,
    hover_image_id: Option<image::Id>,
    press_image_id: Option<image::Id>,
    color: Option<Color>,
    length: Option<f32>,
}

widget_ids! {
    struct Ids {
        track,
        slider,
        soft_max_slider,
    }
}

/// Represents the state of the ImageSlider widget.
pub struct State {
    ids: Ids,
}

impl<T, K> ImageSlider<T, K> {
    builder_methods! {
        pub skew { skew = f32 }
        pub soft_max { soft_max = T }
        pub pad_track { track.padding = (f32, f32) }
        pub hover_image { slider.hover_image_id = Some(image::Id) }
        pub press_image { slider.press_image_id = Some(image::Id) }
        pub track_breadth { track.breadth = Some(f32) }
        pub slider_length { slider.length = Some(f32) }
        pub track_color { track.color = Some(Color) }
        pub slider_color { slider.color = Some(Color) }
    }

    fn new(value: T, min: T, max: T, slider_image_id: image::Id, track_image_id: image::Id) -> Self
    where
        T: Copy,
    {
        Self {
            common: widget::CommonBuilder::default(),
            value,
            min,
            max,
            soft_max: max,
            skew: 1.0,
            track: Track {
                image_id: track_image_id,
                color: None,
                breadth: None,
                padding: (0.0, 0.0),
            },
            slider: Slider {
                image_id: slider_image_id,
                hover_image_id: None,
                press_image_id: None,
                color: None,
                length: None,
            },
            kind: std::marker::PhantomData,
        }
    }
}

impl<T> ImageSlider<T, Continuous>
where
    T: Float,
{
    pub fn continuous(
        value: T,
        min: T,
        max: T,
        slider_image_id: image::Id,
        track_image_id: image::Id,
    ) -> Self {
        ImageSlider::new(value, min, max, slider_image_id, track_image_id)
    }
}

impl<T> ImageSlider<T, Discrete>
where
    T: Integer + Copy,
{
    pub fn discrete(
        value: T,
        min: T,
        max: T,
        slider_image_id: image::Id,
        track_image_id: image::Id,
    ) -> Self {
        ImageSlider::new(value, min, max, slider_image_id, track_image_id)
    }
}

impl<T: Float> ValueFromPercent<T> for Continuous {
    fn value_from_percent(percent: f32, min: T, max: T) -> T {
        utils::value_from_perc(percent, min, max)
    }
}
impl<T: Integer + NumCast> ValueFromPercent<T> for Discrete {
    fn value_from_percent(percent: f32, min: T, max: T) -> T {
        NumCast::from(
            utils::value_from_perc(percent, min.to_f32().unwrap(), max.to_f32().unwrap()).round(),
        )
        .unwrap()
    }
}

impl<T, K> Widget for ImageSlider<T, K>
where
    T: NumCast + Num + Copy + PartialOrd,
    K: ValueFromPercent<T>,
{
    type Event = Option<T>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {}

    /// Update the state of the Slider.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            rect,
            ui,
            ..
        } = args;
        let ImageSlider {
            value,
            min,
            max,
            skew,
            track,
            slider,
            ..
        } = self;
        let (start_pad, end_pad) = (track.padding.0 as f64, track.padding.1 as f64);

        let is_horizontal = rect.w() > rect.h();

        // Touch screens have no mouse, so the slider also follows a finger
        // that is on it: a drag follows the finger and a tap picks the
        // tapped position on release. Swipes, which scroll the page the
        // slider sits on, are cancelled before they reach the widget, so a
        // touch that is still delivered here belongs to the slider. Where a
        // touch begins is deliberately ignored: it is not known yet whether
        // the finger will tap the slider or swipe the page, and the latter
        // must not change the value. Conrod reports touch positions relative
        // to the centre of the widget, so add the centre back to get absolute
        // coordinates.
        let touch_abs_xy = {
            let input = ui.widget_input(id);
            let center = rect.xy();
            let mut abs_xy = None;
            for event in input.events() {
                if let conrod_core::event::Widget::Touch(touch) = event
                    && matches!(touch.phase, Phase::Move | Phase::End)
                {
                    abs_xy = Some([center[0] + touch.xy[0], center[1] + touch.xy[1]]);
                }
            }
            abs_xy
        };

        // The value a position in absolute coordinates points at.
        let value_from_abs_xy = |abs_xy: Point| {
            let (offset, dim) = if is_horizontal {
                // Horizontal
                (abs_xy[0] - rect.x.start, rect.w())
            } else {
                // Vertical
                (abs_xy[1] - rect.y.start, rect.h())
            };
            let offset = offset - start_pad;
            let track_length = dim - start_pad - end_pad;
            let perc = utils::clamp(offset, 0.0, track_length) / track_length;
            let skewed_perc = (perc).powf(skew as f64);
            K::value_from_percent(skewed_perc as f32, min, max)
        };

        let new_value = if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                value_from_abs_xy(mouse.abs_xy())
            } else {
                value
            }
        } else if let Some(abs_xy) = touch_abs_xy {
            value_from_abs_xy(abs_xy)
        } else {
            value
        };

        // Track
        let track_rect = if is_horizontal {
            let h = slider.length.map_or(rect.h() / 3.0, |h| h as f64);
            Rect {
                y: Range::from_pos_and_len(rect.y(), h),
                ..rect
            }
        } else {
            let w = slider.length.map_or(rect.w() / 3.0, |w| w as f64);
            Rect {
                x: Range::from_pos_and_len(rect.x(), w),
                ..rect
            }
        };

        let (x, y, w, h) = track_rect.x_y_w_h();
        Image::new(track.image_id)
            .x_y(x, y)
            .w_h(w, h)
            .parent(id)
            .graphics_for(id)
            .color(track.color)
            .set(state.ids.track, ui);

        // Slider
        // A finger dragging the slider or lifting off it presses the slider,
        // just like the mouse's left button does.
        let mouse_down = ui
            .widget_input(id)
            .mouse()
            .is_some_and(|mouse| mouse.buttons.left().is_down());
        let slider_image = if mouse_down || touch_abs_xy.is_some() {
            slider
                .press_image_id
                .or(slider.hover_image_id)
                .unwrap_or(slider.image_id)
        } else if ui.widget_input(id).mouse().is_some() {
            slider.hover_image_id.unwrap_or(slider.image_id)
        } else {
            slider.image_id
        };

        // A rectangle for positioning and sizing the slider.
        let slider_rect = |slider_value| {
            let value_perc = utils::map_range(slider_value, min, max, 0.0, 1.0);
            let unskewed_perc = value_perc.powf(1.0 / skew as f64);
            if is_horizontal {
                let pos = utils::map_range(
                    unskewed_perc,
                    0.0,
                    1.0,
                    rect.x.start + start_pad,
                    rect.x.end - end_pad,
                );
                let w = slider.length.map_or(rect.w() / 10.0, |w| w as f64);
                Rect {
                    x: Range::from_pos_and_len(pos, w),
                    ..rect
                }
            } else {
                let pos = utils::map_range(
                    unskewed_perc,
                    0.0,
                    1.0,
                    rect.y.start + start_pad,
                    rect.y.end - end_pad,
                );
                let h = slider.length.map_or(rect.h() / 10.0, |h| h as f64);
                Rect {
                    y: Range::from_pos_and_len(pos, h),
                    ..rect
                }
            }
        };

        // Whether soft max slider needs to be displayed and main slider faded to look
        // like a ghost.
        let over_soft_max = new_value > self.soft_max;

        let (x, y, w, h) = slider_rect(new_value).x_y_w_h();
        let fade = if over_soft_max { 0.5 } else { 1.0 };
        Image::new(slider_image)
            .x_y(x, y)
            .w_h(w, h)
            .parent(id)
            .graphics_for(id)
            .color(Some(
                slider
                    .color
                    .map_or(Color::Rgba(1.0, 1.0, 1.0, fade), |c: Color| c.alpha(fade)),
            ))
            .set(state.ids.slider, ui);

        if over_soft_max {
            let (x, y, w, h) = slider_rect(self.soft_max).x_y_w_h();
            Image::new(slider_image)
                .x_y(x, y)
                .w_h(w, h)
                .parent(id)
                .graphics_for(id)
                .color(slider.color)
                .set(state.ids.soft_max_slider, ui);
        }

        // If the value has just changed, return the new value.
        if value != new_value {
            Some(new_value)
        } else {
            None
        }
    }
}

impl<T, K> Colorable for ImageSlider<T, K> {
    fn color(mut self, color: Color) -> Self {
        self.slider.color = Some(color);
        self.track.color = Some(color);
        self
    }
}
