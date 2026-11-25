//! The central English string catalog (FR-027). Every user-visible string the engine
//! produces is a key here; `text` is the only place literals live.

/// Keys for user-visible text. Templates use `{pos}`, `{cols}`, and `{rows}` placeholders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringKey {
    BadgeReady,
    BadgeValue,
    BadgeLabel,
    BadgeEdit,
    BadgePoint,
    BadgeMenu,
    BadgeDialog,
    DiagSyntax,
    DiagUseQuoteForText,
    EnlargeWindow,
}

pub fn text(key: StringKey) -> &'static str {
    match key {
        StringKey::BadgeReady => "READY",
        StringKey::BadgeValue => "VALUE",
        StringKey::BadgeLabel => "LABEL",
        StringKey::BadgeEdit => "EDIT",
        StringKey::BadgePoint => "POINT",
        StringKey::BadgeMenu => "MENU",
        StringKey::BadgeDialog => "DIALOG",
        StringKey::DiagSyntax => "Formula syntax error at character {pos}; the entry was not stored",
        StringKey::DiagUseQuoteForText => "To store text that starts with a digit, begin it with an apostrophe (')",
        StringKey::EnlargeWindow => "Enlarge the window to at least {cols}x{rows}",
    }
}
