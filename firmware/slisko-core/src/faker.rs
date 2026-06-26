//! Ported from `pkg/faker` (Blinker, Interval, RandomBlinker, RandomInterval).
//!
//! The Go fakers hold a `time.Time` start and call `time.Since(st)`, and pull
//! randomness from the global `math/rand`. Here every faker is driven by an
//! explicit elapsed-seconds clock (`trig(now)`), and the randomized ones own a
//! seeded [`Rng`] so rendering needs no shared RNG. Durations are `f32` seconds.

use alloc::boxed::Box;

use crate::utils;

/// Small seedable PRNG (wraps `oorandom`). Mirrors `utils.Random` /
/// `utils.RandomInt64`.
#[derive(Clone)]
pub struct Rng(oorandom::Rand32);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(oorandom::Rand32::new(seed))
    }

    /// Uniform `f32` in `[min, max)` (mirrors `utils.Random`).
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.0.rand_float() * (max - min)
    }

    /// Uniform `i64`-ish in `[min, max)` (mirrors `utils.RandomInt64`).
    pub fn range_secs(&mut self, min: f32, max: f32) -> f32 {
        self.range_f32(min, max)
    }

    /// Spawn a fresh seed for a child faker so each gets an independent stream.
    pub fn next_seed(&mut self) -> u64 {
        let hi = self.0.rand_u32() as u64;
        let lo = self.0.rand_u32() as u64;
        (hi << 32) | lo
    }
}

/// Equivalent of the Go `Fake` interface.
pub trait Fake {
    /// Current value in `[0, 1]` at elapsed time `now` (seconds).
    fn trig(&mut self, now: f32) -> f32;
}

/// `Blinker`: a square wave of a sine (mirrors `faker.Blinker`).
pub struct Blinker {
    st: f32,
    speed: f32,
}

impl Blinker {
    pub fn new(speed: f32, now: f32) -> Self {
        Blinker { st: now, speed }
    }
}

impl Fake for Blinker {
    fn trig(&mut self, now: f32) -> f32 {
        utils::square(libm::sinf(self.speed * (now - self.st)))
    }
}

/// `RandomBlinker`: a sine whose speed re-randomizes on a random interval; the
/// output is a `DutyCycle` of that sine (mirrors `faker.RandomBlinker`).
pub struct RandomBlinker {
    st: f32,
    iv: f32,
    speed: f32,
    min_speed: f32,
    max_speed: f32,
    min_time: f32,
    max_time: f32,
    rng: Rng,
}

impl RandomBlinker {
    pub fn new(
        min_speed: f32,
        max_speed: f32,
        min_time: f32,
        max_time: f32,
        now: f32,
        mut rng: Rng,
    ) -> Self {
        let iv = rng.range_secs(min_time, max_time);
        let speed = rng.range_f32(min_speed, max_speed);
        RandomBlinker {
            st: now,
            iv,
            speed,
            min_speed,
            max_speed,
            min_time,
            max_time,
            rng,
        }
    }
}

impl Fake for RandomBlinker {
    fn trig(&mut self, now: f32) -> f32 {
        if now - self.st > self.iv {
            self.iv = self.rng.range_secs(self.min_time, self.max_time);
            self.speed = self.rng.range_f32(self.min_speed, self.max_speed);
            self.st = now;
        }
        utils::duty_cycle(libm::sinf(self.speed * (now - self.st)), 0.80)
    }
}

/// `RandomInterval`: gates an inner faker on a random on/off interval (mirrors
/// `faker.RandomInterval`).
pub struct RandomInterval {
    blink: Box<dyn Fake + Send>,
    st: f32,
    iv: f32,
    bl: f32,
    min_interval: f32,
    max_interval: f32,
    min_blink: f32,
    max_blink: f32,
    rng: Rng,
}

impl RandomInterval {
    pub fn new(
        min_i: f32,
        max_i: f32,
        min_b: f32,
        max_b: f32,
        blink: Box<dyn Fake + Send>,
        now: f32,
        mut rng: Rng,
    ) -> Self {
        let iv = rng.range_secs(min_i, max_i);
        let bl = rng.range_secs(min_b, max_b);
        RandomInterval {
            blink,
            st: now,
            iv,
            bl,
            min_interval: min_i,
            max_interval: max_i,
            min_blink: min_b,
            max_blink: max_b,
            rng,
        }
    }
}

impl Fake for RandomInterval {
    fn trig(&mut self, now: f32) -> f32 {
        if now - self.st > self.iv + self.bl {
            self.iv = self.rng.range_secs(self.min_interval, self.max_interval);
            self.bl = self.rng.range_secs(self.min_blink, self.max_blink);
            self.st = now;
        }
        if now - self.st > self.iv {
            return self.blink.trig(now);
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinker_is_bounded() {
        let mut b = Blinker::new(2.0, 0.0);
        for i in 0..100 {
            let v = b.trig(i as f32 * 0.05);
            assert!(v == 0.0 || v == 1.0);
        }
    }

    #[test]
    fn random_blinker_stays_in_unit_range() {
        let mut b = RandomBlinker::new(15.0, 40.0, 1.0, 10.0, 0.0, Rng::new(42));
        for i in 0..200 {
            let v = b.trig(i as f32 * 0.1);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn random_interval_gates_inner() {
        let inner = Box::new(Blinker::new(20.0, 0.0));
        let mut ri = RandomInterval::new(0.05, 7.0, 0.05, 12.0, inner, 0.0, Rng::new(7));
        for i in 0..200 {
            let v = ri.trig(i as f32 * 0.05);
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
