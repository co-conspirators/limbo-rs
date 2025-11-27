use iced::id::Id;
use iced::widget::{mouse_area, row, text};

use crate::components::{icon, section};
use crate::message::Message;

#[derive(Debug, Clone, Copy)]
enum TimeFormat {
    _12h,
    _24h,
}

pub struct Clock {
    id: Id,
    now: jiff::Zoned,
    format: TimeFormat,
    expanded: bool,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            id: Id::unique(),
            now: jiff::Zoned::now(),
            // TODO: make configurable
            format: TimeFormat::_12h,
            expanded: false,
        }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::ClockTick(now) => {
                self.now = now.clone();
            }
            Message::ClockToggleExpanded(id) if *id == self.id => {
                self.expanded = !self.expanded;
                self.now = jiff::Zoned::now();
            }
            _ => {}
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let format = match (self.format, self.expanded) {
            // Sun 5:14 PM
            (TimeFormat::_12h, false) => "%a %-I:%M %p",
            // Sunday, Jun 22 5:14:34 PM
            (TimeFormat::_12h, true) => "%A, %b %d %H:%M:%S %p",
            // Sun 22:14
            (TimeFormat::_24h, false) => "%a %k:%M",
            // Sunday, Jun 22 22:14:34
            (TimeFormat::_24h, true) => "%A, %b %e %k:%M:%S",
        };
        let formatted_date = self.now.strftime(format).to_string();

        mouse_area(section(
            row![icon("clock", None), text(formatted_date)].spacing(8),
        ))
        .on_press(Message::ClockToggleExpanded(self.id.clone()))
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.expanded {
            time::every_second()
        } else {
            time::every_minute()
        }
        .map(|_| Message::ClockTick(jiff::Zoned::now()))
    }
}

/// Returns an [`iced::Subscription`] that produces a message at every integer second
/// or minute transition, such that the clock is updated exactly on time.
pub mod time {
    use iced::Subscription;
    use iced::advanced::subscription::{self, Hasher};

    pub fn every_second() -> Subscription<()> {
        subscription::from_recipe(Timer { seconds: true })
    }

    pub fn every_minute() -> Subscription<()> {
        subscription::from_recipe(Timer { seconds: false })
    }

    struct Timer {
        seconds: bool,
    }

    impl subscription::Recipe for Timer {
        type Output = ();

        fn hash(&self, state: &mut Hasher) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
            self.seconds.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: subscription::EventStream,
        ) -> iced::futures::stream::BoxStream<'static, Self::Output> {
            use iced::futures::stream::StreamExt;

            let seconds = self.seconds;
            let stream = iced::futures::stream::unfold((), move |_| async move {
                let now_sys = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system time is valid");

                let delay = if seconds {
                    // Wait until next second
                    std::time::Duration::from_nanos(1_000_000_000 - now_sys.subsec_nanos() as u64)
                } else {
                    // Wait until next minute
                    let secs_into_minute = now_sys.as_secs() % 60;
                    let nanos = now_sys.subsec_nanos() as u64;
                    std::time::Duration::from_nanos((60 - secs_into_minute) * 1_000_000_000 - nanos)
                };

                tokio::time::sleep(delay).await;
                Some(((), ()))
            });

            stream.boxed()
        }
    }
}
