use std::time::Duration;

use color::{AlphaColor, Lab, Srgb};
use iced::Color;

use crate::message::Message;

/// Interval in milliseconds between `Message::AnimationTick` events.
const ANIMATION_TICKRATE: u64 = (1000. / 60.) as u64;

pub trait Lerpable {
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

#[derive(Debug, Clone, Copy)]
pub struct EasedToggle<V> {
    easing: Easing,
    speed: f32,
    target: bool,
    progress: f32,
    /// Value to attain when target is false
    f: V,
    /// Value to attain when target is true
    t: V,
}

impl<V: Lerpable> EasedToggle<V> {
    pub fn new(target: bool, easing: Easing, duration: f32, f: V, t: V) -> Self {
        Self {
            easing,
            speed: ANIMATION_TICKRATE as f32 / duration,
            target,
            progress: if target { 1.0 } else { 0.0 },
            f,
            t,
        }
    }

    pub fn with_target(mut self, target: bool) -> Self {
        self.target = target;
        self
    }

    pub fn set_target(&mut self, target: bool) {
        self.target = target;
    }

    pub fn is_running(&self) -> bool {
        self.target && self.progress < 1.0 || !self.target && self.progress > 0.0
    }

    pub fn update(&mut self) {
        if self.target && self.progress < 1.0 {
            self.progress = (self.progress + self.speed).min(1.0);
        } else if !self.target && self.progress > 0.0 {
            self.progress = (self.progress - self.speed).max(0.0);
        }
    }

    pub fn get(&self) -> V {
        let factor = self.easing.ease(self.progress);
        Lerpable::lerp(&self.f, &self.t, factor)
    }
}

pub fn subscription() -> iced::Subscription<Message> {
    iced::time::every(Duration::from_millis(ANIMATION_TICKRATE)).map(|_| Message::AnimationTick)
}
