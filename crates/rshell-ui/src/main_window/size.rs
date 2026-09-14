use gtk::{gdk, prelude::*};

const FALLBACK: (i32, i32) = (1_360, 860);
const MINIMUM: (i32, i32) = (960, 640);
const MAXIMUM: (i32, i32) = (1_920, 1_200);
const MONITOR_MARGIN: i32 = 48;

fn floor_to_spacing(value: i32) -> i32 {
    value / 4 * 4
}

fn initial_window_size(monitor_size: Option<(i32, i32)>) -> (i32, i32) {
    let Some((width, height)) = monitor_size else {
        return FALLBACK;
    };
    let fit = |dimension: i32, minimum: i32, maximum: i32| {
        let target = floor_to_spacing(dimension.saturating_mul(80) / 100);
        let available = floor_to_spacing((dimension - MONITOR_MARGIN).max(0));
        target.max(minimum).min(maximum).min(available)
    };
    (
        fit(width, MINIMUM.0, MAXIMUM.0),
        fit(height, MINIMUM.1, MAXIMUM.1),
    )
}

pub(crate) fn startup_window_size() -> (i32, i32) {
    let monitor_size = gdk::Display::default()
        .and_then(|display| display.monitors().item(0))
        .and_then(|monitor| monitor.downcast::<gdk::Monitor>().ok())
        .map(|monitor| {
            let geometry = monitor.geometry();
            (geometry.width(), geometry.height())
        });
    initial_window_size(monitor_size)
}

#[cfg(test)]
mod tests {
    use super::initial_window_size;

    #[test]
    fn startup_size_scales_caps_and_preserves_small_screen_margin() {
        assert_eq!(initial_window_size(Some((2_560, 1_600))), (1_920, 1_200));
        assert_eq!(initial_window_size(Some((10_000, 8_000))), (1_920, 1_200));
        assert_eq!(initial_window_size(Some((1_366, 768))), (1_092, 640));
        assert_eq!(initial_window_size(Some((1_280, 800))), (1_024, 640));
        assert_eq!(initial_window_size(Some((800, 600))), (752, 552));
    }

    #[test]
    fn startup_size_retains_previous_default_without_monitor_geometry() {
        assert_eq!(initial_window_size(None), (1_360, 860));
    }
}
