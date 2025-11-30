use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use iced::widget::svg::{Handle, Svg};
use iced::widget::{Container, container, image, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::icons::{Icons, IconsFilled};

pub fn side<'a, Message>(
    alignment: Alignment,
    content: impl Into<iced::Element<'a, Message>>,
) -> Container<'a, Message> {
    container(content).width(Length::Fill).align_x(alignment)
}

pub fn text_with_icon<'a, Message: 'a>(
    _icon: &'a str,
    color: Option<Color>,
    _text: impl text::IntoFragment<'a>,
) -> iced::Element<'a, Message> {
    let icon = icon(_icon, color);
    let text = text(_text);
    row![icon, text].spacing(6).into()
}

static ICON_CACHE: LazyLock<Mutex<HashMap<String, Option<PathBuf>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub fn system_icon<'a, Message>(name: &str) -> Option<Element<'a, Message>> {
    let icon_path = ICON_CACHE
        .lock()
        .ok()?
        .entry(name.to_string())
        .or_insert_with(|| freedesktop_icons::lookup(name).with_size(48).find())
        .clone()?;
    Some(image(image::Handle::from_path(icon_path)).into())
}

pub fn icon(name: &'_ str, color: Option<Color>) -> Svg<'static> {
    svg(Handle::from_memory(
        Icons::get(&format!("{name}.svg")).unwrap().data,
    ))
    .style(move |theme: &Theme, _| svg::Style {
        color: color.or(Some(theme.palette().text)),
    })
    .width(Length::Shrink)
    .height(Length::Fixed(16.))
}

pub fn icon_filled(name: &'_ str, color: Option<Color>) -> Svg<'static> {
    svg(Handle::from_memory(
        IconsFilled::get(&format!("{name}.svg")).unwrap().data,
    ))
    .style(move |theme: &Theme, _| svg::Style {
        color: color.or(Some(theme.palette().text)),
    })
    .width(Length::Shrink)
}

impl crate::config::Config {
    pub fn section<'a, Message>(
        &self,
        content: impl Into<iced::Element<'a, Message>>,
    ) -> Container<'a, Message> {
        let background = Some(iced::Background::Color(
            self.theme
                .resolve_color(&self.bar.theme.section_bg)
                .unwrap_or(Color::parse("#2c2c3f").unwrap()),
        ));
        let radius = iced::Radius::new(self.theme.border_radius);

        container(content)
            .style(move |_| container::Style {
                background,
                border: Border {
                    radius,
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([6, 12])
            .align_y(Alignment::Center)
            .height(Length::Fill)
    }

    pub fn icon(&self, _icon: &crate::config::types::Icon) -> Svg<'static> {
        icon(&_icon.name, self.theme.resolve_color(&_icon.color))
    }

    pub fn text_with_icon<'a, Message: 'a>(
        &self,
        _icon: &'a crate::config::types::Icon,
        _text: impl text::IntoFragment<'a>,
    ) -> iced::Element<'a, Message> {
        text_with_icon(&_icon.name, self.theme.resolve_color(&_icon.color), _text)
    }
}
