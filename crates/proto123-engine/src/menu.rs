//! Slash Command Bar state machine, command dispatch, and catalog verification (US5, FR-025/026).

use crate::command::{CommandId, CommandSpec};
use crate::menu_catalog::{CATALOG, MenuKind, MenuNode};
use crate::session::{Mode, Session};
use crate::strings::text;
use std::collections::HashSet;
use std::fmt;

/// One violation found by [`verify`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuViolation {
    pub path: String,
    pub message: String,
}

impl fmt::Display for MenuViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Checks every item in `catalog` against `registry` and trigger/string invariants (SC-011).
pub fn verify(catalog: &[MenuNode], registry: &[CommandSpec]) -> Vec<MenuViolation> {
    let mut violations = Vec::new();
    verify_level(catalog, registry, "", &mut violations);
    violations
}

fn verify_level(
    nodes: &[MenuNode],
    registry: &[CommandSpec],
    prefix: &str,
    out: &mut Vec<MenuViolation>,
) {
    let mut seen_triggers = HashSet::new();
    for node in nodes {
        let label = text(node.label);
        let desc = text(node.desc);
        let path = if prefix.is_empty() {
            format!("/{label}")
        } else {
            format!("{prefix}/{label}")
        };
        if label.trim().is_empty() {
            out.push(MenuViolation {
                path: path.clone(),
                message: "empty menu label string".to_owned(),
            });
        }
        if desc.trim().is_empty() {
            out.push(MenuViolation {
                path: path.clone(),
                message: "empty menu description string".to_owned(),
            });
        }
        let trigger_upper = node.trigger.to_ascii_uppercase();
        if !seen_triggers.insert(trigger_upper) {
            out.push(MenuViolation {
                path: path.clone(),
                message: format!("duplicate trigger '{trigger_upper}' at this level"),
            });
        }
        let first_char = label.chars().next().map(|c| c.to_ascii_uppercase());
        if first_char != Some(trigger_upper) {
            out.push(MenuViolation {
                path: path.clone(),
                message: format!(
                    "trigger '{trigger_upper}' does not match first letter of {label:?}"
                ),
            });
        }
        match node.kind {
            MenuKind::Branch(children) => {
                if children.is_empty() {
                    out.push(MenuViolation {
                        path: path.clone(),
                        message: "branch has no child items".to_owned(),
                    });
                } else {
                    verify_level(children, registry, &path, out);
                }
            }
            MenuKind::Leaf(id) => {
                if !registry.iter().any(|spec| spec.id == id) {
                    out.push(MenuViolation {
                        path: path.clone(),
                        message: format!("unregistered command id {id:?}"),
                    });
                }
            }
        }
    }
}

/// Active menu position (FR-025).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    pub path: Vec<usize>,
    pub highlighted: usize,
    pub hovered: Option<usize>,
}

impl MenuState {
    pub fn active_index(&self) -> usize {
        self.hovered.unwrap_or(self.highlighted)
    }

    pub fn current_level<'a>(&self, root: &'a [MenuNode]) -> &'a [MenuNode] {
        let mut level = root;
        for &idx in &self.path {
            let Some(node) = level.get(idx) else {
                return level;
            };
            match node.kind {
                MenuKind::Branch(children) => level = children,
                MenuKind::Leaf(_) => return level,
            }
        }
        level
    }

    pub fn active_node<'a>(&self, root: &'a [MenuNode]) -> Option<&'a MenuNode> {
        let level = self.current_level(root);
        level.get(self.active_index())
    }

    /// Second-line text: submenu item list for a branch, or description for a leaf (FR-025).
    pub fn second_line_text(&self, root: &[MenuNode]) -> String {
        let Some(node) = self.active_node(root) else {
            return String::new();
        };
        second_line_for_node(node)
    }
}

pub fn second_line_for_node(node: &MenuNode) -> String {
    match node.kind {
        MenuKind::Branch(children) => children
            .iter()
            .map(|c| text(c.label))
            .collect::<Vec<_>>()
            .join("  "),
        MenuKind::Leaf(_) => text(node.desc).to_owned(),
    }
}

impl Session {
    pub(crate) fn menu_open(&mut self) {
        if self.mode() != Mode::Ready {
            return;
        }
        self.set_root_hover(None);
        self.set_menu(Some(MenuState::default()));
        self.set_mode(Mode::Menu);
    }

    pub(crate) fn menu_open_at(&mut self, root_index: usize) -> bool {
        if !matches!(self.mode(), Mode::Ready | Mode::Menu) {
            return false;
        }
        let Some(node) = CATALOG.get(root_index) else {
            return false;
        };
        self.set_root_hover(None);
        match node.kind {
            MenuKind::Branch(_) => {
                self.set_menu(Some(MenuState {
                    path: vec![root_index],
                    highlighted: 0,
                    hovered: None,
                }));
                self.set_mode(Mode::Menu);
                false
            }
            MenuKind::Leaf(id) => {
                self.set_menu(None);
                self.set_mode(Mode::Ready);
                self.run_menu_leaf(id)
            }
        }
    }

    pub(crate) fn menu_move(&mut self, delta: i8) {
        if self.mode() != Mode::Menu {
            return;
        }
        let Some(mut menu) = self.menu().cloned() else {
            return;
        };
        let len = menu.current_level(CATALOG).len();
        if len == 0 {
            return;
        }
        let cur = menu.highlighted.min(len - 1);
        let next = if delta < 0 {
            (cur + len - 1) % len
        } else if delta > 0 {
            (cur + 1) % len
        } else {
            cur
        };
        menu.highlighted = next;
        menu.hovered = None;
        self.set_menu(Some(menu));
    }

    pub(crate) fn menu_hover(&mut self, index: Option<usize>) {
        match self.mode() {
            Mode::Ready => {
                let valid = index.filter(|&i| i < CATALOG.len());
                self.set_root_hover(valid);
            }
            Mode::Menu => {
                let Some(mut menu) = self.menu().cloned() else {
                    return;
                };
                let len = menu.current_level(CATALOG).len();
                menu.hovered = index.filter(|&i| i < len);
                self.set_menu(Some(menu));
            }
            _ => {}
        }
    }

    pub(crate) fn menu_activate(&mut self, index: Option<usize>) -> bool {
        if self.mode() != Mode::Menu {
            return false;
        }
        let Some(mut menu) = self.menu().cloned() else {
            return false;
        };
        let level = menu.current_level(CATALOG);
        let target = index.unwrap_or_else(|| menu.active_index());
        let Some(node) = level.get(target) else {
            return false;
        };
        match node.kind {
            MenuKind::Branch(_) => {
                menu.path.push(target);
                menu.highlighted = 0;
                menu.hovered = None;
                self.set_menu(Some(menu));
                false
            }
            MenuKind::Leaf(id) => {
                self.set_menu(None);
                self.set_mode(Mode::Ready);
                self.run_menu_leaf(id)
            }
        }
    }

    pub(crate) fn menu_trigger(&mut self, ch: char) -> bool {
        if self.mode() != Mode::Menu {
            return false;
        }
        let Some(menu) = self.menu().cloned() else {
            return false;
        };
        let level = menu.current_level(CATALOG);
        let Some(idx) = level
            .iter()
            .position(|node| node.trigger.eq_ignore_ascii_case(&ch))
        else {
            return false;
        };
        self.menu_activate(Some(idx))
    }

    pub(crate) fn menu_back(&mut self) {
        if self.mode() != Mode::Menu {
            return;
        }
        let Some(mut menu) = self.menu().cloned() else {
            self.set_mode(Mode::Ready);
            return;
        };
        if menu.path.pop().is_some() {
            menu.highlighted = 0;
            menu.hovered = None;
            self.set_menu(Some(menu));
        } else {
            self.set_menu(None);
            self.set_mode(Mode::Ready);
        }
    }

    pub(crate) fn menu_close(&mut self) {
        if self.mode() == Mode::Menu {
            self.set_menu(None);
            self.set_mode(Mode::Ready);
        }
        self.set_root_hover(None);
    }

    fn run_menu_leaf(&mut self, id: CommandId) -> bool {
        match id {
            CommandId::FileNewWorkbook => self.file_new_workbook(),
            CommandId::EditUndo => self.undo(),
            CommandId::EditRedo => self.redo(),
            CommandId::RangeErase => self.erase_selection(),
            CommandId::RangeCopy => {
                self.open_copy_prompt();
                false
            }
            CommandId::RangeMove => {
                self.open_move_prompt();
                false
            }
            CommandId::ViewCycleTheme => {
                self.cycle_theme();
                false
            }
            _ => false,
        }
    }
}
