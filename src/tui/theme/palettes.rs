impl EditorTheme {
    pub(crate) fn for_id(theme_id: EditorThemeId) -> Self {
        match theme_id {
            EditorThemeId::Nocturne => Self {
                header_fg: Color::White,
                header_bg: Color::DarkBlue,
                gutter_fg: Color::DarkGrey,
                status_fg: Color::Black,
                status_bg: Color::Cyan,
                search_fg: Color::Rgb { r: 0, g: 0, b: 0 },
                search_bg: Color::Rgb {
                    r: 90,
                    g: 230,
                    b: 245,
                },
                help_fg: Color::Grey,
                help_bg: Color::Black,
                dirty_fg: Color::Yellow,
            },
            EditorThemeId::Aurora => Self {
                header_fg: Color::Black,
                header_bg: Color::Green,
                gutter_fg: Color::DarkCyan,
                status_fg: Color::Black,
                status_bg: Color::Magenta,
                search_fg: Color::Rgb { r: 0, g: 0, b: 0 },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 120,
                    b: 220,
                },
                help_fg: Color::Cyan,
                help_bg: Color::Black,
                dirty_fg: Color::Yellow,
            },
            EditorThemeId::Paper => Self {
                header_fg: Color::Rgb {
                    r: 245,
                    g: 226,
                    b: 244,
                },
                header_bg: Color::Rgb {
                    r: 118,
                    g: 67,
                    b: 169,
                },
                gutter_fg: Color::Rgb {
                    r: 155,
                    g: 48,
                    b: 96,
                },
                status_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                status_bg: Color::Rgb {
                    r: 236,
                    g: 180,
                    b: 224,
                },
                search_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 207,
                    b: 119,
                },
                help_fg: Color::Rgb {
                    r: 34,
                    g: 24,
                    b: 48,
                },
                help_bg: Color::Rgb {
                    r: 245,
                    g: 226,
                    b: 244,
                },
                dirty_fg: Color::Rgb {
                    r: 139,
                    g: 83,
                    b: 31,
                },
            },
            EditorThemeId::Terminal => Self {
                header_fg: Color::Rgb {
                    r: 168,
                    g: 255,
                    b: 168,
                },
                header_bg: Color::Rgb { r: 0, g: 36, b: 12 },
                gutter_fg: Color::DarkGreen,
                status_fg: Color::Black,
                status_bg: Color::Rgb {
                    r: 72,
                    g: 255,
                    b: 112,
                },
                search_fg: Color::Rgb { r: 0, g: 20, b: 7 },
                search_bg: Color::Rgb {
                    r: 154,
                    g: 255,
                    b: 104,
                },
                help_fg: Color::Rgb {
                    r: 114,
                    g: 215,
                    b: 132,
                },
                help_bg: Color::Black,
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 228,
                    b: 92,
                },
            },
            EditorThemeId::Abyss => Self {
                header_fg: Color::Rgb {
                    r: 102,
                    g: 229,
                    b: 255,
                },
                header_bg: Color::Rgb { r: 8, g: 15, b: 32 },
                gutter_fg: Color::Rgb {
                    r: 72,
                    g: 88,
                    b: 122,
                },
                status_fg: Color::Rgb {
                    r: 206,
                    g: 240,
                    b: 255,
                },
                status_bg: Color::Rgb {
                    r: 18,
                    g: 33,
                    b: 58,
                },
                search_fg: Color::Rgb { r: 4, g: 12, b: 26 },
                search_bg: Color::Rgb {
                    r: 112,
                    g: 236,
                    b: 255,
                },
                help_fg: Color::Rgb {
                    r: 141,
                    g: 188,
                    b: 205,
                },
                help_bg: Color::Rgb { r: 3, g: 7, b: 18 },
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 64,
                    b: 96,
                },
            },
            EditorThemeId::Terror => Self {
                header_fg: Color::Rgb {
                    r: 255,
                    g: 42,
                    b: 160,
                },
                header_bg: Color::Rgb { r: 45, g: 0, b: 58 },
                gutter_fg: Color::Rgb {
                    r: 166,
                    g: 80,
                    b: 190,
                },
                status_fg: Color::Rgb {
                    r: 255,
                    g: 188,
                    b: 236,
                },
                status_bg: Color::Rgb { r: 78, g: 0, b: 92 },
                search_fg: Color::Rgb { r: 31, g: 0, b: 36 },
                search_bg: Color::Rgb {
                    r: 255,
                    g: 75,
                    b: 184,
                },
                help_fg: Color::Rgb {
                    r: 255,
                    g: 88,
                    b: 190,
                },
                help_bg: Color::Rgb { r: 24, g: 0, b: 30 },
                dirty_fg: Color::Rgb {
                    r: 255,
                    g: 238,
                    b: 70,
                },
            },
        }
    }
}
