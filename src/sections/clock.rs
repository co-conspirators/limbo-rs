use crate::components::section;

pub struct Clock {
    now: jiff::Zoned,
}

#[derive(Debug, Clone)]
pub enum ClockMessage {
    Tick(jiff::Zoned),
}

impl Clock {
    pub fn new() -> Self {
        Self {
            now: jiff::Zoned::now(),
        }
    }

    pub fn update(&mut self, message: ClockMessage) {
        match message {
            ClockMessage::Tick(local_time) => {
                self.now = local_time;
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, ClockMessage> {
        section(iced::widget::text(
            self.now.strftime("%B %d, %X").to_string(),
        ))
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<ClockMessage> {
        time::every_second().map(|_| ClockMessage::Tick(jiff::Zoned::now()))
    }
}

pub mod time {
    use iced::{
        Subscription,
        advanced::subscription::{self, Hasher},
    };

    /// Returns an [`iced::Subscription`] that produces a message at every integer second
    /// transition.
    pub fn every_second() -> Subscription<std::time::Instant> {
        subscription::from_recipe(EverySecond())
    }

    struct EverySecond();

    impl subscription::Recipe for EverySecond {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut Hasher) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: subscription::EventStream,
        ) -> iced::futures::stream::BoxStream<'static, Self::Output> {
            use iced::futures::stream::StreamExt;

            let now = tokio::time::Instant::now();
            let now_sys = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is valid");

            let subsec_nanos = now_sys.subsec_nanos() as u64;
            let until_next_second = std::time::Duration::from_secs(1)
                .checked_sub(std::time::Duration::from_nanos(subsec_nanos))
                .unwrap_or_default();

            let mut interval = tokio::time::interval_at(
                now + until_next_second,
                tokio::time::Duration::from_secs(1),
            );
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            let stream = iced::futures::stream::unfold(interval, |mut interval| async move {
                Some((interval.tick().await.into_std(), interval))
            });

            stream.boxed()
        }
    }
}
