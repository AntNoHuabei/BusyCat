use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct PhoneExpression {
    seed: u64,
    active: bool,
    next: Instant,
    started: Instant,
    index: usize,
}
impl Default for PhoneExpression {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                | 1,
            active: false,
            next: now,
            started: now,
            index: 0,
        }
    }
}
impl PhoneExpression {
    fn random(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }
    pub fn sample(&mut self, phone: bool, now: Instant) -> Option<(usize, f32)> {
        if !phone {
            self.active = false;
            return None;
        }
        if !self.active {
            self.active = true;
            self.next = now + Duration::from_millis(900 + self.random() % 1300);
            self.started = now - Duration::from_secs(10);
        }
        if now >= self.next {
            self.index = (self.index + 1 + (self.random() % 4) as usize) % 5;
            self.started = now;
            self.next = now + Duration::from_millis(4200 + self.random() % 3500);
        }
        let age = now.saturating_duration_since(self.started).as_secs_f32();
        // Short reaction, then back to quietly scrolling. Crossfade the face only.
        let alpha = (age / 0.18).min((2.6 - age) / 0.35).clamp(0.0, 1.0);
        (alpha > 0.0).then_some((self.index, alpha))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reactions_are_spaced_do_not_repeat_and_stop_on_input() {
        let t = Instant::now();
        let mut s = PhoneExpression {
            seed: 17,
            ..Default::default()
        };
        assert!(s.sample(true, t).is_none());
        let mut last = None;
        for _ in 0..20 {
            let next = s.next;
            s.sample(true, next);
            let (i, a) = s.sample(true, next + Duration::from_millis(300)).unwrap();
            assert_eq!(a, 1.0);
            assert_ne!(last, Some(i));
            last = Some(i);
            assert!(s.sample(true, next + Duration::from_secs(3)).is_none());
        }
        assert!(s.sample(false, s.next).is_none());
        assert!(s.sample(true, s.next + Duration::from_secs(1)).is_none());
    }
}
