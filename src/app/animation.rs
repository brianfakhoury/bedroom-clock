//! Fade timing, in milliseconds since the display was shown.

pub const DISPLAY_DURATION_MS: u32 = 3000;
pub const FADE_IN_MS: u32 = 180;
pub const FADE_OUT_MS: u32 = 350;

const FADE_OUT_START_MS: u32 = DISPLAY_DURATION_MS - FADE_OUT_MS;

// Cubic smoothstep, fixed point
fn ease(elapsed: u32, duration: u32) -> u32 {
    if elapsed >= duration {
        return 1000;
    }
    let x = elapsed * 1000 / duration;
    x * x * (3000 - 2 * x) / 1_000_000
}

/// Brightness at `ms`, fading in from `from` to `target` and out to zero
pub fn level(ms: u32, from: u32, target: u32) -> u32 {
    if ms >= DISPLAY_DURATION_MS {
        0
    } else if ms < FADE_IN_MS {
        let f = ease(ms, FADE_IN_MS);
        (from * (1000 - f) + target * f) / 1000
    } else if ms < FADE_OUT_START_MS {
        target
    } else {
        target * (1000 - ease(ms - FADE_OUT_START_MS, FADE_OUT_MS)) / 1000
    }
}

/// Delay until the next frame in milliseconds; zero when finished
pub fn next_ms(ms: u32) -> u32 {
    if ms >= DISPLAY_DURATION_MS {
        0
    } else if !(FADE_IN_MS..FADE_OUT_START_MS).contains(&ms) {
        10
    } else {
        FADE_OUT_START_MS - ms
    }
}
