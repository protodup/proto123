//! Static Slash Command Bar catalog (FR-026, contracts/commands.md "Menu tree").
//! Checked by `menu::verify` and `cargo xtask menu-commands`.

use crate::command::CommandId;
use crate::strings::StringKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
    Branch(&'static [MenuNode]),
    Leaf(CommandId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuNode {
    pub label: StringKey,
    pub desc: StringKey,
    pub trigger: char,
    pub kind: MenuKind,
}

const FILE_ITEMS: &[MenuNode] = &[MenuNode {
    label: StringKey::MenuFileNew,
    desc: StringKey::MenuFileNewDesc,
    trigger: 'N',
    kind: MenuKind::Leaf(CommandId::FileNewWorkbook),
}];

const EDIT_ITEMS: &[MenuNode] = &[
    MenuNode {
        label: StringKey::MenuEditUndo,
        desc: StringKey::MenuEditUndoDesc,
        trigger: 'U',
        kind: MenuKind::Leaf(CommandId::EditUndo),
    },
    MenuNode {
        label: StringKey::MenuEditRedo,
        desc: StringKey::MenuEditRedoDesc,
        trigger: 'R',
        kind: MenuKind::Leaf(CommandId::EditRedo),
    },
];

const RANGE_ITEMS: &[MenuNode] = &[
    MenuNode {
        label: StringKey::MenuRangeErase,
        desc: StringKey::MenuRangeEraseDesc,
        trigger: 'E',
        kind: MenuKind::Leaf(CommandId::RangeErase),
    },
    MenuNode {
        label: StringKey::MenuRangeCopy,
        desc: StringKey::MenuRangeCopyDesc,
        trigger: 'C',
        kind: MenuKind::Leaf(CommandId::RangeCopy),
    },
    MenuNode {
        label: StringKey::MenuRangeMove,
        desc: StringKey::MenuRangeMoveDesc,
        trigger: 'M',
        kind: MenuKind::Leaf(CommandId::RangeMove),
    },
];

const VIEW_ITEMS: &[MenuNode] = &[MenuNode {
    label: StringKey::MenuViewTheme,
    desc: StringKey::MenuViewThemeDesc,
    trigger: 'T',
    kind: MenuKind::Leaf(CommandId::ViewCycleTheme),
}];

/// Exact Spec 001 menu tree (FR-026).
pub const CATALOG: &[MenuNode] = &[
    MenuNode {
        label: StringKey::MenuFile,
        desc: StringKey::MenuFileDesc,
        trigger: 'F',
        kind: MenuKind::Branch(FILE_ITEMS),
    },
    MenuNode {
        label: StringKey::MenuEdit,
        desc: StringKey::MenuEditDesc,
        trigger: 'E',
        kind: MenuKind::Branch(EDIT_ITEMS),
    },
    MenuNode {
        label: StringKey::MenuRange,
        desc: StringKey::MenuRangeDesc,
        trigger: 'R',
        kind: MenuKind::Branch(RANGE_ITEMS),
    },
    MenuNode {
        label: StringKey::MenuView,
        desc: StringKey::MenuViewDesc,
        trigger: 'V',
        kind: MenuKind::Branch(VIEW_ITEMS),
    },
];
