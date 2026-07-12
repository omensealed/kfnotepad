//! Theme-specific terminal syntax palettes.

use super::*;

pub(super) fn terminal_syntax_role_rgb(
    theme_id: EditorThemeId,
    role: TerminalSyntaxColorRole,
) -> (u8, u8, u8) {
    match theme_id {
        EditorThemeId::Nocturne => match role {
            TerminalSyntaxColorRole::Text => (213, 224, 246),
            TerminalSyntaxColorRole::Comment => (126, 141, 170),
            TerminalSyntaxColorRole::Rose => (255, 126, 154),
            TerminalSyntaxColorRole::Orange => (246, 177, 116),
            TerminalSyntaxColorRole::Yellow => (238, 213, 122),
            TerminalSyntaxColorRole::Green => (139, 222, 160),
            TerminalSyntaxColorRole::Cyan => (99, 221, 224),
            TerminalSyntaxColorRole::Blue => (132, 172, 255),
            TerminalSyntaxColorRole::Purple => (202, 158, 255),
        },
        EditorThemeId::Aurora => match role {
            TerminalSyntaxColorRole::Text => (218, 255, 241),
            TerminalSyntaxColorRole::Comment => (112, 162, 156),
            TerminalSyntaxColorRole::Rose => (255, 129, 162),
            TerminalSyntaxColorRole::Orange => (255, 183, 112),
            TerminalSyntaxColorRole::Yellow => (245, 224, 128),
            TerminalSyntaxColorRole::Green => (104, 241, 151),
            TerminalSyntaxColorRole::Cyan => (65, 234, 217),
            TerminalSyntaxColorRole::Blue => (119, 198, 255),
            TerminalSyntaxColorRole::Purple => (208, 151, 255),
        },
        EditorThemeId::Paper => match role {
            TerminalSyntaxColorRole::Text => (80, 67, 91),
            TerminalSyntaxColorRole::Comment => (119, 105, 130),
            TerminalSyntaxColorRole::Rose => (154, 62, 100),
            TerminalSyntaxColorRole::Orange => (158, 87, 48),
            TerminalSyntaxColorRole::Yellow => (125, 94, 20),
            TerminalSyntaxColorRole::Green => (45, 116, 93),
            TerminalSyntaxColorRole::Cyan => (37, 111, 126),
            TerminalSyntaxColorRole::Blue => (67, 89, 153),
            TerminalSyntaxColorRole::Purple => (118, 72, 156),
        },
        EditorThemeId::Terminal => match role {
            TerminalSyntaxColorRole::Text => (168, 255, 176),
            TerminalSyntaxColorRole::Comment => (83, 165, 95),
            TerminalSyntaxColorRole::Rose => (255, 126, 126),
            TerminalSyntaxColorRole::Orange => (247, 186, 96),
            TerminalSyntaxColorRole::Yellow => (240, 250, 127),
            TerminalSyntaxColorRole::Green => (80, 255, 119),
            TerminalSyntaxColorRole::Cyan => (113, 255, 207),
            TerminalSyntaxColorRole::Blue => (142, 215, 255),
            TerminalSyntaxColorRole::Purple => (205, 168, 255),
        },
        EditorThemeId::Abyss => match role {
            TerminalSyntaxColorRole::Text => (214, 244, 255),
            TerminalSyntaxColorRole::Comment => (100, 132, 158),
            TerminalSyntaxColorRole::Rose => (255, 97, 137),
            TerminalSyntaxColorRole::Orange => (255, 169, 111),
            TerminalSyntaxColorRole::Yellow => (241, 218, 111),
            TerminalSyntaxColorRole::Green => (111, 230, 172),
            TerminalSyntaxColorRole::Cyan => (93, 239, 255),
            TerminalSyntaxColorRole::Blue => (126, 174, 255),
            TerminalSyntaxColorRole::Purple => (196, 145, 255),
        },
        EditorThemeId::Terror => match role {
            TerminalSyntaxColorRole::Text => (255, 193, 238),
            TerminalSyntaxColorRole::Comment => (157, 103, 148),
            TerminalSyntaxColorRole::Rose => (255, 62, 166),
            TerminalSyntaxColorRole::Orange => (255, 120, 75),
            TerminalSyntaxColorRole::Yellow => (255, 226, 82),
            TerminalSyntaxColorRole::Green => (91, 255, 141),
            TerminalSyntaxColorRole::Cyan => (90, 238, 230),
            TerminalSyntaxColorRole::Blue => (136, 172, 255),
            TerminalSyntaxColorRole::Purple => (221, 97, 255),
        },
    }
}
