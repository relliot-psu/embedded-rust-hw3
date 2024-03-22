use crate::*;

type RgbPins = [Output<'static, AnyPin>; 3];

pub struct Rgb {
    rgb: RgbPins,
    // Shadow variables to minimize lock contention.
    levels: [u32; 3],
    tick_time: u64,
}

impl Rgb {
    /// Compute the duration of a tick (in microseconds, for use in [`Rgb::step`]),
    /// such that complete cycles through the 3 components happen at the given
    /// `frame_rate` (in hertz).
    fn frame_tick_time(frame_rate: u64) -> u64 {
        1_000_000 / (3 * frame_rate * LEVELS as u64)
    }

    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self {
            rgb,
            levels: [0; 3],
            tick_time,
        }
    }

    /// Given a specific LED RGB component (`{0: red, 1: blue, 2: green}`),
    /// using the current `level` for the component, turn it on for `level` ticks,
    /// and off for ([`LEVELS`]-`level`) ticks, for a total duration [LEVELS] ticks.
    async fn step(&mut self, led: usize) {
        let level = self.levels[led];
        if level > 0 {
            self.rgb[led].set_high();
            let on_time = level as u64 * self.tick_time;
            Timer::after_micros(on_time).await;
            self.rgb[led].set_low();
        }
        let inverse_level = LEVELS - level;
        if inverse_level > 0 {
            let off_time = inverse_level as u64 * self.tick_time;
            Timer::after_micros(off_time).await;
        }
    }

    pub async fn run(mut self) -> ! {
        loop {
            self.levels = get_rgb_levels().await;

            for led in 0..3 {
                self.step(led).await;
            }
        }
    }
}
