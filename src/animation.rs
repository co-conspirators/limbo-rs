use std::time::Duration;

use color::{AlphaColor, Lab, Srgb};
use iced::Color;

use crate::message::Message;

/// Interval in milliseconds between `Message::AnimationTick` events.
const ANIMATION_TICKRATE: u64 = (1000. / 60.) as u64;

pub trait Lerpable: Clone + Copy {
    fn lerp(start: &Self, end: &Self, factor: f32) -> Self;
}

impl Lerpable for f32 {
    fn lerp(start: &Self, end: &Self, factor: f32) -> Self {
        start + factor * (end - start)
    }
}

impl Lerpable for Color {
    fn lerp(start: &Self, end: &Self, factor: f32) -> Self {
        fn iced2color(c: &Color) -> AlphaColor<Lab> {
            AlphaColor::<Srgb>::new([c.r, c.g, c.b, c.a]).convert()
        }

        let start = iced2color(start);
        let end = iced2color(end);
        let color = start.lerp(end, factor, color::HueDirection::Shorter);
        let color: AlphaColor<Srgb> = color.convert();
        let c = color.components;
        Color {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    Smoothstep,
}

impl Easing {
    pub fn ease(self, x: f32) -> f32 {
        match self {
            Self::Linear => x,
            Self::Smoothstep => x * x * (3.0 - 2.0 * x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eased<V> {
    easing: Easing,
    /// Amount to increment progress by, per tick.
    speed: f32,
    /// The currently desired target index.
    target_idx: usize,
    /// 0..=1 progress between `start` and `end`.
    progress: f32,
    /// Start value of the currently running animation.
    start: V,
    /// End value of the currently running animation.
    end: V,
    /// All possible target values; `end` is always one of these.
    targets: Box<[V]>,
}

impl<V: Lerpable> Eased<V> {
    pub fn new(initial_target_idx: usize, easing: Easing, duration: f32, targets: &[V]) -> Self {
        assert!(
            !targets.is_empty(),
            "targets must contain at least one element"
        );
        assert!(
            initial_target_idx < targets.len(),
            "initial_target_idx out of range"
        );

        Self {
            easing,
            speed: ANIMATION_TICKRATE as f32 / duration,
            target_idx: initial_target_idx,
            progress: 1.0,
            start: targets[initial_target_idx],
            end: targets[initial_target_idx],
            targets: targets.into(),
        }
    }

    pub fn with_target_idx(mut self, target_idx: usize) -> Self {
        self.set_target_idx(target_idx);
        self
    }

    /// Set a new target index and start an animation from the *current* interpolated value.
    pub fn set_target_idx(&mut self, target_idx: usize) {
        assert!(target_idx < self.targets.len(), "target_idx out of range");

        // If the new target equals the old target and we're already settled, nothing to do.
        if target_idx == self.target_idx && (self.progress >= 1.0) {
            return;
        }

        self.start = self.get();
        self.end = self.targets[target_idx];
        self.progress = 0.0;
        self.target_idx = target_idx;
    }

    pub fn is_running(&self) -> bool {
        self.progress < 1.0
    }

    pub fn update(&mut self) {
        if self.progress < 1.0 {
            self.progress = (self.progress + self.speed).min(1.0);
        }
    }

    pub fn get(&self) -> V {
        let factor = self.easing.ease(self.progress);
        Lerpable::lerp(&self.start, &self.end, factor)
    }
}

pub fn subscription() -> iced::Subscription<Message> {
    iced::time::every(Duration::from_millis(ANIMATION_TICKRATE)).map(|_| Message::AnimationTick)
}
