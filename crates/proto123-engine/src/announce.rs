//! Accessibility live-region announcement builder (FR-036, contracts/wasm-bridge.md).

use crate::menu_catalog::{CATALOG, MenuKind};
use crate::session::{Mode, Session};
use crate::strings::{StringKey, text};
use proto123_model::CellContent;
use proto123_value::{Value, format_full};

fn format_value_full(v: &Value) -> String {
    match v {
        Value::Empty => String::new(),
        Value::Number(n) => format_full(*n),
        Value::Text(s) => s.clone(),
        Value::Bool(true) => "TRUE".to_owned(),
        Value::Bool(false) => "FALSE".to_owned(),
        Value::Error(e) => e.token().to_owned(),
    }
}

fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_lowercase().to_string();
    out.push_str(chars.as_str());
    out
}

pub(crate) fn build_announcement(session: &Session) -> String {
    let mode_str = text(session.mode().badge());
    let base = if session.mode() == Mode::Menu
        && let Some(menu) = session.menu()
        && let Some(node) = menu.active_node(CATALOG)
    {
        let item = text(node.label);
        let raw_desc = match node.kind {
            MenuKind::Branch(_) => text(node.desc).to_owned(),
            MenuKind::Leaf(_) => lower_first(text(node.desc)),
        };
        if let Some(&root_idx) = menu.path.last()
            && let Some(parent_node) = CATALOG.get(root_idx)
        {
            text(StringKey::AnnounceMenuSub)
                .replace("{parent}", text(parent_node.label))
                .replace("{item}", item)
                .replace("{desc}", &raw_desc)
                .replace("{mode}", mode_str)
        } else {
            text(StringKey::AnnounceMenuRoot)
                .replace("{item}", item)
                .replace("{desc}", &raw_desc)
                .replace("{mode}", mode_str)
        }
    } else if let Some(idx) = session.root_hover()
        && let Some(node) = CATALOG.get(idx)
    {
        let raw_desc = match node.kind {
            MenuKind::Branch(_) => text(node.desc).to_owned(),
            MenuKind::Leaf(_) => lower_first(text(node.desc)),
        };
        text(StringKey::AnnounceMenuRoot)
            .replace("{item}", text(node.label))
            .replace("{desc}", &raw_desc)
            .replace("{mode}", mode_str)
    } else {
        let addr = session.active().to_string();
        match session.calc().content(session.active()) {
            Some(CellContent::Formula(f)) => {
                let val = format_value_full(&session.calc().value(session.active()));
                text(StringKey::AnnounceCellFormula)
                    .replace("{addr}", &addr)
                    .replace("{formula}", &f.canonical)
                    .replace("{value}", &val)
                    .replace("{mode}", mode_str)
            }
            Some(CellContent::Number(_)) => {
                let val = format_value_full(&session.calc().value(session.active()));
                text(StringKey::AnnounceCellValue)
                    .replace("{addr}", &addr)
                    .replace("{value}", &val)
                    .replace("{mode}", mode_str)
            }
            Some(CellContent::Label {
                text: label_text, ..
            }) => text(StringKey::AnnounceCellLabel)
                .replace("{addr}", &addr)
                .replace("{label}", label_text)
                .replace("{mode}", mode_str),
            None => text(StringKey::AnnounceCellEmpty)
                .replace("{addr}", &addr)
                .replace("{mode}", mode_str),
        }
    };

    match session.status() {
        Some(status) => text(StringKey::AnnounceWithStatus)
            .replace("{base}", &base)
            .replace("{status}", &status.message()),
        None => base,
    }
}
