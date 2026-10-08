use crate::ui::touch_scroll::TouchScrollTracker;
use iced::{
    Clipboard, Element, Event, Hasher, Layout, Length, Point, Rectangle, Widget, layout, mouse,
    touch,
};
use std::hash::Hash;
use vek::Vec2;

fn tracker_position(position: Point) -> Vec2<f64> {
    Vec2::new(f64::from(position.x), f64::from(position.y))
}

/// State for a [`TouchScrollable`] gesture.
#[derive(Default)]
pub struct TouchScrollState {
    touch: TouchScrollTracker<touch::Finger>,
}

/// Lets a vertical touch drag scroll content even when it starts on an
/// interactive child, such as a button in a scrollable list.
pub struct TouchScrollable<'a, Message, Renderer: iced::Renderer> {
    content: Element<'a, Message, Renderer>,
    state: &'a mut TouchScrollState,
}

impl<'a, Message, Renderer> TouchScrollable<'a, Message, Renderer>
where
    Renderer: iced::Renderer,
{
    pub fn new(
        state: &'a mut TouchScrollState,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            state,
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for TouchScrollable<'_, Message, Renderer>
where
    Renderer: iced::Renderer,
{
    fn width(&self) -> Length { self.content.width() }

    fn height(&self) -> Length { self.content.height() }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.layout(renderer, limits)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        self.content
            .draw(renderer, defaults, layout, cursor_position, viewport)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.content.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> iced::event::Status {
        let bounds = layout.bounds();
        let event = match event {
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if bounds.contains(position) {
                    self.state.touch.start(id, tracker_position(position));
                }
                Event::Touch(touch::Event::FingerPressed { id, position })
            },
            Event::Touch(touch::Event::FingerMoved { id, position }) => {
                if let Some(delta) = self.state.touch.move_to(id, tracker_position(position)) {
                    // Don't also send the touch-move event: iced's Scrollable
                    // handles touch drags that start on empty space itself, but
                    // a child button captures the initial press. A wheel event
                    // scrolls the parent in either case without double-scrolling.
                    Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels {
                            x: 0.0,
                            y: -delta.y as f32,
                        },
                    })
                } else {
                    Event::Touch(touch::Event::FingerMoved { id, position })
                }
            },
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                if self.state.touch.finish(id) {
                    // A swipe must not activate the button where it started.
                    Event::Touch(touch::Event::FingerLost { id, position })
                } else {
                    Event::Touch(touch::Event::FingerLifted { id, position })
                }
            },
            Event::Touch(touch::Event::FingerLost { id, position }) => {
                self.state.touch.finish(id);
                Event::Touch(touch::Event::FingerLost { id, position })
            },
            event => event,
        };

        self.content.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<iced::overlay::Element<'_, Message, Renderer>> {
        self.content.overlay(layout)
    }
}

impl<'a, Message, Renderer> From<TouchScrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + iced::Renderer,
    Message: 'a,
{
    fn from(widget: TouchScrollable<'a, Message, Renderer>) -> Self { Element::new(widget) }
}
