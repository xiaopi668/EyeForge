use iced::color;

#[derive(Debug, Clone, Copy, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    pub fn bg_main(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x2d, 0x2d, 0x2d),
            Self::Light => color!(0xf5, 0xf5, 0xf5),
        }
    }

    pub fn bg_sidebar(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x25, 0x25, 0x25),
            Self::Light => color!(0xe8, 0xe8, 0xe8),
        }
    }

    pub fn fg(self) -> iced::Color {
        match self {
            Self::Dark => color!(0xd4, 0xd4, 0xd4),
            Self::Light => color!(0x33, 0x33, 0x33),
        }
    }

    pub fn accent(self) -> iced::Color {
        color!(0x00, 0xd4, 0xaa)
    }

    pub fn sidebar_sel(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x3c, 0x3c, 0x3c),
            Self::Light => color!(0xe0, 0xe0, 0xe0),
        }
    }

    pub fn input_bg(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x3c, 0x3c, 0x3c),
            Self::Light => color!(0xff, 0xff, 0xff),
        }
    }

    pub fn border(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x55, 0x55, 0x55),
            Self::Light => color!(0xcc, 0xcc, 0xcc),
        }
    }

    pub fn btn_bg(self) -> iced::Color {
        match self {
            Self::Dark => color!(0x3c, 0x3c, 0x3c),
            Self::Light => color!(0xe0, 0xe0, 0xe0),
        }
    }
}

// StyleSheet 已通过 main.rs 的 theme 函数实现，此处不再需要
