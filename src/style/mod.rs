pub mod icon_raw;


use iced::{button, container, pick_list};

use iced::{Background, Color};

pub struct ContainerStyle;
impl container::StyleSheet for ContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: None,
            border_radius: 5.0,
            border_width: 2.0,
            border_color: Color {
                r: 0.33,
                g: 0.33,
                b: 0.34,
                a: 1.0,
            },
        }
    }
}

pub struct ButtonStyle;
impl button::StyleSheet for ButtonStyle {
    fn active(&self) -> button::Style {
        // 文本颜色随着主题而变化
        button::Style {
            background: Some(Background::Color(Color {
                r: 0.79,
                g: 0.90,
                b: 1.0,
                a: 1.0,
            })),
            border_radius: 5.0,
            border_width: 1.0,
            // 边框颜色置为透明
            border_color: Color {
                r: 0.94,
                g: 0.55,
                b: 0.55,
                a: 1.0,
            },
            text_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },

            // 这个语法之前提到过了，Rust会自动将未指定的项设置的和..后的结构体的值一致
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 2.0),
            ..active
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::default(),
            ..self.active()
        }
    }
}

pub struct Picklist;

impl pick_list::StyleSheet for Picklist {
    fn menu(&self) -> pick_list::Menu {
        pick_list::Menu {
            background: Background::Color(Color {
                r: 0.79,
                g: 0.90,
                b: 1.0,
                a: 1.0,
            }),
            ..Default::default()
        }
    }

    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            background: Background::Color(Color {
                r: 0.79,
                g: 0.90,
                b: 1.0,
                a: 1.0,
            }),
            border_radius: 5.0,
            border_width: 1.0,
            // 边框颜色置为透明
            border_color: Color {
                r: 0.94,
                g: 0.55,
                b: 0.55,
                a: 1.0,
            },
            text_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },

            // 这个语法之前提到过了，Rust会自动将未指定的项设置的和..后的结构体的值一致
            ..Default::default()
        }
    }

    fn hovered(&self) -> pick_list::Style {
        pick_list::Style {
            border_color: Color::BLACK,
            ..self.active()
        }
    }
}
