use iced::{
    Background, Border, Color, Shadow, Theme,
    border::Radius,
    widget::{
        button, pick_list,
        slider::{self, HandleShape},
    },
};
use std::cell::LazyCell;

pub const PRIMARY_COLOR: LazyCell<Color> = LazyCell::new(|| Color::parse("#1281E6").unwrap());
pub const LIGHT_PRIMARY_COLOR: LazyCell<Color> = LazyCell::new(|| Color::parse("#348fe3").unwrap());
pub const ERROR_COLOR: Color = Color::from_rgba(1.0, 0.333, 0.333, 0.8);
pub const BACKGROUND_COLOR: LazyCell<Color> = LazyCell::new(|| Color::parse("#2E2E2E").unwrap());
pub const DARKER_BACKGROUND_COLOR: LazyCell<Color> =
    LazyCell::new(|| Color::parse("#272727").unwrap());

pub fn picklist() -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style {
    |_, _| pick_list::Style {
        text_color: Color::WHITE,
        placeholder_color: Color::WHITE,
        handle_color: Color::WHITE,
        background: Background::Color(*DARKER_BACKGROUND_COLOR),
        border: Border {
            color: *PRIMARY_COLOR,
            width: 1.0,
            radius: Radius::new(8),
        },
    }
}

pub fn refresh_btn() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_, _| button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: *PRIMARY_COLOR,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn volume_slider() -> impl Fn(&Theme, slider::Status) -> slider::Style {
    |_, _| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(*PRIMARY_COLOR),
                Background::Color(Color::WHITE),
            ),
            width: 3.0,
            border: Border::default().rounded(5),
        },
        handle: slider::Handle {
            shape: HandleShape::Rectangle {
                width: 8,
                border_radius: Radius::new(10),
            },
            background: Background::Color(*PRIMARY_COLOR),
            border_width: 1.0,
            border_color: *PRIMARY_COLOR,
        },
    }
}

pub fn volume_slider_muted() -> impl Fn(&Theme, slider::Status) -> slider::Style {
    |_, _| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(ERROR_COLOR),
                Background::Color(ERROR_COLOR),
            ),
            width: 3.0,
            border: Border::default().rounded(5),
        },
        handle: slider::Handle {
            shape: HandleShape::Circle { radius: 1.5 },
            background: Background::Color(ERROR_COLOR),
            border_width: 0.0,
            border_color: ERROR_COLOR,
        },
    }
}

pub fn generic_button() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_, status| match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(*PRIMARY_COLOR)),
            text_color: Color::WHITE,
            border: Border::default().rounded(6),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(*LIGHT_PRIMARY_COLOR)),
            text_color: Color::WHITE,
            border: Border::default().rounded(6),
            shadow: Shadow::default(),
        },
    }
}
