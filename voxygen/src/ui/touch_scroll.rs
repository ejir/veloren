use vek::Vec2;

/// Turns a finger drag into incremental scroll deltas for UI toolkits that only
/// expose wheel scrolling on their scrollable widgets.
pub(super) struct TouchScrollTracker<I> {
    active: Option<ActiveTouch<I>>,
}

struct ActiveTouch<I> {
    id: I,
    origin: Vec2<f64>,
    last: Vec2<f64>,
    moved: bool,
    scrolling: bool,
}

impl<I> Default for TouchScrollTracker<I> {
    fn default() -> Self { Self { active: None } }
}

impl<I: Copy + PartialEq> TouchScrollTracker<I> {
    // Android UI units are much smaller than density-independent pixels, so
    // use a larger threshold there to avoid turning ordinary tap jitter into a
    // cancelled button press.
    const DRAG_THRESHOLD: f64 = if cfg!(target_os = "android") {
        20.0
    } else {
        8.0
    };

    pub fn start(&mut self, id: I, position: Vec2<f64>) {
        // Keep the first finger as the scroll gesture. Additional fingers are
        // commonly used for gameplay gestures and should not change the target.
        if self.active.is_none() {
            self.active = Some(ActiveTouch {
                id,
                origin: position,
                last: position,
                moved: false,
                scrolling: false,
            });
        }
    }

    /// Return a scroll delta once the movement is large enough to be a swipe,
    /// rather than a button tap. Positions use the UI toolkit's coordinate
    /// system so the delta is already in UI units.
    pub fn move_to(&mut self, id: I, position: Vec2<f64>) -> Option<Vec2<f64>> {
        let touch = self.active.as_mut().filter(|touch| touch.id == id)?;
        let total_delta = position - touch.origin;
        if total_delta.magnitude_squared() >= Self::DRAG_THRESHOLD * Self::DRAG_THRESHOLD {
            touch.moved = true;
        }
        let started_scrolling = !touch.scrolling
            && total_delta.y.abs() >= Self::DRAG_THRESHOLD
            && total_delta.y.abs() >= total_delta.x.abs();

        if started_scrolling {
            touch.scrolling = true;
        }

        let delta = if started_scrolling {
            total_delta
        } else {
            position - touch.last
        };
        touch.last = position;

        touch.scrolling.then_some(delta)
    }

    /// End the gesture and report whether the finger moved far enough to
    /// cancel a tap. Scroll gestures must be cancelled in the UI toolkit so
    /// releasing over the original widget does not also activate it.
    pub fn finish(&mut self, id: I) -> bool {
        if self.active.as_ref().is_some_and(|touch| touch.id == id) {
            self.active.take().is_some_and(|touch| touch.moved)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_for_a_drag_before_scrolling_and_cancels_taps() {
        let mut tracker = TouchScrollTracker::default();
        tracker.start(7, Vec2::new(0.0, 0.0));

        let threshold = TouchScrollTracker::<i32>::DRAG_THRESHOLD;
        assert_eq!(tracker.move_to(7, Vec2::new(2.0, 2.0)), None);
        assert!(!tracker.finish(7));

        tracker.start(7, Vec2::new(0.0, 0.0));
        assert_eq!(
            tracker.move_to(7, Vec2::new(0.0, threshold + 2.0)),
            Some(Vec2::new(0.0, threshold + 2.0))
        );
        assert_eq!(
            tracker.move_to(7, Vec2::new(0.0, threshold + 8.0)),
            Some(Vec2::new(0.0, 6.0))
        );
        assert!(tracker.finish(7));
    }

    #[test]
    fn horizontal_drags_do_not_scroll_but_do_not_activate_the_start_widget() {
        let mut tracker = TouchScrollTracker::default();
        tracker.start(3, Vec2::new(0.0, 0.0));

        assert_eq!(
            tracker.move_to(
                3,
                Vec2::new(TouchScrollTracker::<i32>::DRAG_THRESHOLD + 4.0, 1.0)
            ),
            None
        );
        assert!(tracker.finish(3));
    }

    #[test]
    fn ignores_other_fingers() {
        let mut tracker = TouchScrollTracker::default();
        tracker.start(1, Vec2::new(0.0, 0.0));

        assert_eq!(
            tracker.move_to(
                2,
                Vec2::new(0.0, TouchScrollTracker::<i32>::DRAG_THRESHOLD + 4.0)
            ),
            None
        );
        assert!(!tracker.finish(2));
        assert!(!tracker.finish(1));
    }
}
