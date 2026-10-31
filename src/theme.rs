use ratatui::style::Color;

/// The colour of each role of the list; a role's colour is the same wherever it is drawn.
#[derive(PartialEq)]
pub struct Theme {
    /// Painted on every unset cell so the list and the popup's frame read as one.
    pub(crate) background: Color,
    /// Where the keys go: the mode block, the border of the focused card or floating window.
    pub(crate) accent: Color,
    /// Text on the accent.
    pub(crate) on_accent: Color,
    /// Text read first, painted on every unset cell.
    pub(crate) primary: Color,
    /// Text read second: labels, section names, a message, the history.
    pub(crate) secondary: Color,
    /// Labels of the detail zone, a step greyer than the values they name.
    pub(crate) label: Color,
    /// Text read only when looked for: dates, `key:value` words, counts, key actions, done tasks.
    pub(crate) tertiary: Color,
    /// Rules and borders.
    pub(crate) structure: Color,
    /// Background of a floating window, a step above the screen's.
    pub(crate) raised: Color,
    /// Background of the row under the cursor in the zone that gets the keys.
    pub(crate) selected: Color,
    /// Separators between keys, border of a floating window without the keys.
    pub(crate) separator: Color,
    /// Background of a priority badge.
    pub(crate) badge: Color,
    /// A `+project`.
    pub(crate) project: Color,
    /// An `@context`.
    pub(crate) context: Color,
    /// A due date past, or something refused.
    pub(crate) alert: Color,
    /// Something done, told by a toast.
    pub(crate) success: Color,
    /// A due date on the day.
    pub(crate) due_today: Color,
    /// Priorities A to E.
    pub(crate) priorities: [Color; 5],
}

impl Theme {
    /// Catppuccin Mocha, the background being its mantle, herdr's popup colour.
    pub const MOCHA: Theme = Theme {
        background: Color::Rgb(0x18, 0x18, 0x25),
        accent: Color::Rgb(0xfa, 0xb3, 0x87),
        on_accent: Color::Rgb(0x11, 0x11, 0x1b),
        primary: Color::Rgb(0xcd, 0xd6, 0xf4),
        secondary: Color::Rgb(0xa6, 0xad, 0xc8),
        label: Color::Rgb(0x93, 0x99, 0xb2),
        tertiary: Color::Rgb(0x7f, 0x84, 0x9c),
        structure: Color::Rgb(0x45, 0x47, 0x5a),
        raised: Color::Rgb(0x1e, 0x1e, 0x2e),
        selected: Color::Rgb(0x31, 0x32, 0x44),
        separator: Color::Rgb(0x58, 0x5b, 0x70),
        badge: Color::Rgb(0x45, 0x47, 0x5a),
        project: Color::Rgb(0xcb, 0xa6, 0xf7),
        context: Color::Rgb(0x94, 0xe2, 0xd5),
        alert: Color::Rgb(0xf3, 0x8b, 0xa8),
        success: Color::Rgb(0xa6, 0xe3, 0xa1),
        due_today: Color::Rgb(0xf9, 0xe2, 0xaf),
        priorities: [
            Color::Rgb(0xf5, 0xc2, 0xe7),
            Color::Rgb(0xa6, 0xe3, 0xa1),
            Color::Rgb(0x89, 0xb4, 0xfa),
            Color::Rgb(0xb4, 0xbe, 0xfe),
            Color::Rgb(0x89, 0xdc, 0xeb),
        ],
    };

    /// Catppuccin Latte, with the roles of Mocha.
    pub const LATTE: Theme = Theme {
        background: Color::Rgb(0xe6, 0xe9, 0xef),
        accent: Color::Rgb(0xfe, 0x64, 0x0b),
        on_accent: Color::Rgb(0xdc, 0xe0, 0xe8),
        primary: Color::Rgb(0x4c, 0x4f, 0x69),
        secondary: Color::Rgb(0x6c, 0x6f, 0x85),
        label: Color::Rgb(0x7c, 0x7f, 0x93),
        tertiary: Color::Rgb(0x8c, 0x8f, 0xa1),
        structure: Color::Rgb(0xbc, 0xc0, 0xcc),
        raised: Color::Rgb(0xef, 0xf1, 0xf5),
        selected: Color::Rgb(0xcc, 0xd0, 0xda),
        separator: Color::Rgb(0xac, 0xb0, 0xbe),
        badge: Color::Rgb(0xbc, 0xc0, 0xcc),
        project: Color::Rgb(0x88, 0x39, 0xef),
        context: Color::Rgb(0x17, 0x92, 0x99),
        alert: Color::Rgb(0xd2, 0x0f, 0x39),
        success: Color::Rgb(0x40, 0xa0, 0x2b),
        due_today: Color::Rgb(0xdf, 0x8e, 0x1d),
        priorities: [
            Color::Rgb(0xea, 0x76, 0xcb),
            Color::Rgb(0x40, 0xa0, 0x2b),
            Color::Rgb(0x1e, 0x66, 0xf5),
            Color::Rgb(0x72, 0x87, 0xfd),
            Color::Rgb(0x04, 0xa5, 0xe5),
        ],
    };

    /// Catppuccin Frappé, with the roles of Mocha.
    pub const FRAPPE: Theme = Theme {
        background: Color::Rgb(0x29, 0x2c, 0x3c),
        accent: Color::Rgb(0xef, 0x9f, 0x76),
        on_accent: Color::Rgb(0x23, 0x26, 0x34),
        primary: Color::Rgb(0xc6, 0xd0, 0xf5),
        secondary: Color::Rgb(0xa5, 0xad, 0xce),
        label: Color::Rgb(0x94, 0x9c, 0xbb),
        tertiary: Color::Rgb(0x83, 0x8b, 0xa7),
        structure: Color::Rgb(0x51, 0x57, 0x6d),
        raised: Color::Rgb(0x30, 0x34, 0x46),
        selected: Color::Rgb(0x41, 0x45, 0x59),
        separator: Color::Rgb(0x62, 0x68, 0x80),
        badge: Color::Rgb(0x51, 0x57, 0x6d),
        project: Color::Rgb(0xca, 0x9e, 0xe6),
        context: Color::Rgb(0x81, 0xc8, 0xbe),
        alert: Color::Rgb(0xe7, 0x82, 0x84),
        success: Color::Rgb(0xa6, 0xd1, 0x89),
        due_today: Color::Rgb(0xe5, 0xc8, 0x90),
        priorities: [
            Color::Rgb(0xf4, 0xb8, 0xe4),
            Color::Rgb(0xa6, 0xd1, 0x89),
            Color::Rgb(0x8c, 0xaa, 0xee),
            Color::Rgb(0xba, 0xbb, 0xf1),
            Color::Rgb(0x99, 0xd1, 0xdb),
        ],
    };

    /// Catppuccin Macchiato, with the roles of Mocha.
    pub const MACCHIATO: Theme = Theme {
        background: Color::Rgb(0x1e, 0x20, 0x30),
        accent: Color::Rgb(0xf5, 0xa9, 0x7f),
        on_accent: Color::Rgb(0x18, 0x19, 0x26),
        primary: Color::Rgb(0xca, 0xd3, 0xf5),
        secondary: Color::Rgb(0xa5, 0xad, 0xcb),
        label: Color::Rgb(0x93, 0x9a, 0xb7),
        tertiary: Color::Rgb(0x80, 0x87, 0xa2),
        structure: Color::Rgb(0x49, 0x4d, 0x64),
        raised: Color::Rgb(0x24, 0x27, 0x3a),
        selected: Color::Rgb(0x36, 0x3a, 0x4f),
        separator: Color::Rgb(0x5b, 0x60, 0x78),
        badge: Color::Rgb(0x49, 0x4d, 0x64),
        project: Color::Rgb(0xc6, 0xa0, 0xf6),
        context: Color::Rgb(0x8b, 0xd5, 0xca),
        alert: Color::Rgb(0xed, 0x87, 0x96),
        success: Color::Rgb(0xa6, 0xda, 0x95),
        due_today: Color::Rgb(0xee, 0xd4, 0x9f),
        priorities: [
            Color::Rgb(0xf5, 0xbd, 0xe6),
            Color::Rgb(0xa6, 0xda, 0x95),
            Color::Rgb(0x8a, 0xad, 0xf4),
            Color::Rgb(0xb7, 0xbd, 0xf8),
            Color::Rgb(0x91, 0xd7, 0xe3),
        ],
    };

    /// No colour at all, for `NO_COLOR`: every role is the terminal's own.
    pub const NONE: Theme = Theme {
        background: Color::Reset,
        accent: Color::Reset,
        on_accent: Color::Reset,
        primary: Color::Reset,
        secondary: Color::Reset,
        label: Color::Reset,
        tertiary: Color::Reset,
        structure: Color::Reset,
        raised: Color::Reset,
        selected: Color::Reset,
        separator: Color::Reset,
        badge: Color::Reset,
        project: Color::Reset,
        context: Color::Reset,
        alert: Color::Reset,
        success: Color::Reset,
        due_today: Color::Reset,
        priorities: [Color::Reset; 5],
    };

    /// The terminal's own ANSI colours: default text and background, no fill for the selection or the badge.
    pub const TERMINAL: Theme = Theme {
        background: Color::Reset,
        accent: Color::Blue,
        on_accent: Color::Black,
        primary: Color::Reset,
        secondary: Color::Reset,
        label: Color::Reset,
        tertiary: Color::DarkGray,
        structure: Color::DarkGray,
        raised: Color::Reset,
        selected: Color::Reset,
        separator: Color::DarkGray,
        badge: Color::Reset,
        project: Color::Magenta,
        context: Color::Cyan,
        alert: Color::Red,
        success: Color::Green,
        due_today: Color::Yellow,
        priorities: [Color::Green; 5],
    };
}

impl Default for Theme {
    fn default() -> Self {
        Theme::TERMINAL
    }
}
