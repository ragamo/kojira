use ratatui::style::Color;

pub struct Theme {
    pub name: &'static str,
    pub accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub bg: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub highlight: Color,
    pub header_bg: Color,
}

pub const ONE_DARK: Theme = Theme {
    name: "one_dark",
    accent: Color::Rgb(97, 175, 239),
    text: Color::Rgb(171, 178, 191),
    text_dim: Color::Rgb(92, 99, 112),
    border: Color::Rgb(62, 68, 81),
    bg: Color::Rgb(40, 44, 52),
    success: Color::Rgb(152, 195, 121),
    error: Color::Rgb(224, 108, 117),
    warning: Color::Rgb(229, 192, 123),
    info: Color::Rgb(86, 182, 194),
    highlight: Color::Rgb(198, 120, 221),
    header_bg: Color::Rgb(44, 49, 58),
};

pub const ONE_LIGHT: Theme = Theme {
    name: "one_light",
    accent: Color::Rgb(64, 120, 242),
    text: Color::Rgb(56, 58, 66),
    text_dim: Color::Rgb(160, 161, 167),
    border: Color::Rgb(229, 229, 230),
    bg: Color::Rgb(250, 250, 250),
    success: Color::Rgb(80, 161, 79),
    error: Color::Rgb(228, 86, 73),
    warning: Color::Rgb(193, 132, 1),
    info: Color::Rgb(1, 132, 188),
    highlight: Color::Rgb(166, 38, 164),
    header_bg: Color::Rgb(240, 240, 241),
};

pub const CATPPUCCIN: Theme = Theme {
    name: "catppuccin",
    accent: Color::Rgb(137, 180, 250),
    text: Color::Rgb(205, 214, 244),
    text_dim: Color::Rgb(108, 112, 134),
    border: Color::Rgb(69, 71, 90),
    bg: Color::Rgb(24, 24, 37),
    success: Color::Rgb(166, 227, 161),
    error: Color::Rgb(243, 139, 168),
    warning: Color::Rgb(249, 226, 175),
    info: Color::Rgb(148, 226, 213),
    highlight: Color::Rgb(203, 166, 247),
    header_bg: Color::Rgb(49, 50, 68),
};

pub const CATPPUCCIN_LATTE: Theme = Theme {
    name: "catppuccin_latte",
    accent: Color::Rgb(30, 102, 245),
    text: Color::Rgb(76, 79, 105),
    text_dim: Color::Rgb(156, 160, 176),
    border: Color::Rgb(188, 192, 204),
    bg: Color::Rgb(239, 241, 245),
    success: Color::Rgb(64, 160, 43),
    error: Color::Rgb(210, 15, 57),
    warning: Color::Rgb(223, 142, 29),
    info: Color::Rgb(23, 146, 153),
    highlight: Color::Rgb(136, 57, 239),
    header_bg: Color::Rgb(204, 208, 218),
};

pub const TOKYO_NIGHT: Theme = Theme {
    name: "tokyo_night",
    accent: Color::Rgb(122, 162, 247),
    text: Color::Rgb(192, 202, 245),
    text_dim: Color::Rgb(86, 95, 137),
    border: Color::Rgb(65, 72, 104),
    bg: Color::Rgb(26, 27, 38),
    success: Color::Rgb(158, 206, 106),
    error: Color::Rgb(247, 118, 142),
    warning: Color::Rgb(224, 175, 104),
    info: Color::Rgb(125, 207, 255),
    highlight: Color::Rgb(187, 154, 247),
    header_bg: Color::Rgb(36, 40, 59),
};

pub const TOKYO_NIGHT_DAY: Theme = Theme {
    name: "tokyo_night_day",
    accent: Color::Rgb(46, 125, 233),
    text: Color::Rgb(55, 96, 191),
    text_dim: Color::Rgb(137, 144, 179),
    border: Color::Rgb(168, 174, 203),
    bg: Color::Rgb(225, 226, 231),
    success: Color::Rgb(88, 117, 57),
    error: Color::Rgb(245, 42, 101),
    warning: Color::Rgb(140, 108, 62),
    info: Color::Rgb(17, 140, 116),
    highlight: Color::Rgb(120, 71, 189),
    header_bg: Color::Rgb(196, 200, 218),
};

pub const DRACULA: Theme = Theme {
    name: "dracula",
    accent: Color::Rgb(189, 147, 249),
    text: Color::Rgb(248, 248, 242),
    text_dim: Color::Rgb(98, 114, 164),
    border: Color::Rgb(68, 71, 90),
    bg: Color::Rgb(40, 42, 54),
    success: Color::Rgb(80, 250, 123),
    error: Color::Rgb(255, 85, 85),
    warning: Color::Rgb(241, 250, 140),
    info: Color::Rgb(139, 233, 253),
    highlight: Color::Rgb(255, 121, 198),
    header_bg: Color::Rgb(68, 71, 90),
};

pub const NORD: Theme = Theme {
    name: "nord",
    accent: Color::Rgb(136, 192, 208),
    text: Color::Rgb(236, 239, 244),
    text_dim: Color::Rgb(76, 86, 106),
    border: Color::Rgb(67, 76, 94),
    bg: Color::Rgb(46, 52, 64),
    success: Color::Rgb(163, 190, 140),
    error: Color::Rgb(191, 97, 106),
    warning: Color::Rgb(235, 203, 139),
    info: Color::Rgb(143, 188, 187),
    highlight: Color::Rgb(180, 142, 173),
    header_bg: Color::Rgb(59, 66, 82),
};

pub const GRUVBOX: Theme = Theme {
    name: "gruvbox",
    accent: Color::Rgb(215, 153, 33),
    text: Color::Rgb(235, 219, 178),
    text_dim: Color::Rgb(146, 131, 116),
    border: Color::Rgb(80, 73, 69),
    bg: Color::Rgb(40, 40, 40),
    success: Color::Rgb(184, 187, 38),
    error: Color::Rgb(251, 73, 52),
    warning: Color::Rgb(250, 189, 47),
    info: Color::Rgb(131, 165, 152),
    highlight: Color::Rgb(211, 134, 155),
    header_bg: Color::Rgb(60, 56, 54),
};

pub const GRUVBOX_LIGHT: Theme = Theme {
    name: "gruvbox_light",
    accent: Color::Rgb(7, 102, 120),
    text: Color::Rgb(60, 56, 54),
    text_dim: Color::Rgb(146, 131, 116),
    border: Color::Rgb(213, 196, 161),
    bg: Color::Rgb(251, 241, 199),
    success: Color::Rgb(121, 116, 14),
    error: Color::Rgb(157, 0, 6),
    warning: Color::Rgb(181, 118, 20),
    info: Color::Rgb(7, 102, 120),
    highlight: Color::Rgb(143, 63, 113),
    header_bg: Color::Rgb(235, 219, 178),
};

pub const SOLARIZED: Theme = Theme {
    name: "solarized",
    accent: Color::Rgb(38, 139, 210),
    text: Color::Rgb(147, 161, 161),
    text_dim: Color::Rgb(88, 110, 117),
    border: Color::Rgb(88, 110, 117),
    bg: Color::Rgb(0, 43, 54),
    success: Color::Rgb(133, 153, 0),
    error: Color::Rgb(220, 50, 47),
    warning: Color::Rgb(181, 137, 0),
    info: Color::Rgb(42, 161, 152),
    highlight: Color::Rgb(211, 54, 130),
    header_bg: Color::Rgb(7, 54, 66),
};

pub const TERMINAL: Theme = Theme {
    name: "terminal",
    accent: Color::Blue,
    text: Color::Reset,
    text_dim: Color::Gray,
    border: Color::DarkGray,
    bg: Color::Reset,
    success: Color::Green,
    error: Color::LightRed,
    warning: Color::Yellow,
    info: Color::Cyan,
    highlight: Color::Magenta,
    header_bg: Color::Reset,
};

pub const ALL_THEMES: &[&Theme] = &[
    &ONE_DARK,
    &ONE_LIGHT,
    &CATPPUCCIN,
    &CATPPUCCIN_LATTE,
    &TOKYO_NIGHT,
    &TOKYO_NIGHT_DAY,
    &DRACULA,
    &NORD,
    &GRUVBOX,
    &GRUVBOX_LIGHT,
    &SOLARIZED,
    &TERMINAL,
];

pub fn find_theme(name: &str) -> &'static Theme {
    ALL_THEMES
        .iter()
        .find(|t| t.name == name)
        .unwrap_or(&&ONE_DARK)
}
