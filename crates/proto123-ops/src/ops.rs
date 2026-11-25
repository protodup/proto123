use crate::delta::CellDelta;
use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, CellRange, MAX_COL, MAX_ROW};
use proto123_model::CellContent;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Pure atomic spreadsheet mutations. Each variant plans a reversible [`CellDelta`] against
/// a read-only [`CalcSheet`] without mutating it (research R10).
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    SetCell {
        addr: CellAddr,
        content: Option<CellContent>,
    },
    Erase {
        range: CellRange,
    },
    Copy {
        src: CellRange,
        dest: CellRange,
    },
    Move {
        src: CellRange,
        dest: CellRange,
    },
}

impl Op {
    pub fn plan(&self, calc: &CalcSheet) -> CellDelta {
        match self {
            Self::SetCell { addr, content } => plan_set_cell(calc, *addr, content.clone()),
            Self::Erase { range } => plan_erase(calc, *range),
            Self::Copy { src, dest } => plan_copy(calc, *src, *dest),
            Self::Move { src, dest } => plan_move(calc, *src, *dest),
        }
    }
}

fn plan_set_cell(calc: &CalcSheet, addr: CellAddr, after: Option<CellContent>) -> CellDelta {
    let before = calc.content(addr).cloned();
    if before == after {
        return CellDelta::default();
    }
    CellDelta {
        changes: vec![(addr, before, after)],
    }
}

fn plan_erase(calc: &CalcSheet, range: CellRange) -> CellDelta {
    let mut changes = Vec::new();
    calc.sheet().for_each_populated_in(range, |addr, cell| {
        changes.push((addr, Some(cell.content.clone()), None));
    });
    CellDelta { changes }
}

fn plan_copy(calc: &CalcSheet, src: CellRange, dest: CellRange) -> CellDelta {
    let mut captured: HashMap<(u32, u32), CellContent> = HashMap::new();
    calc.sheet().for_each_populated_in(src, |addr, cell| {
        let oc = addr.col().saturating_sub(src.start().col());
        let or = addr.row().saturating_sub(src.start().row());
        captured.insert((oc, or), cell.content.clone());
    });

    let tiles_x = if dest.width() >= src.width() {
        (dest.width() / src.width()).max(1)
    } else {
        1
    };
    let tiles_y = if dest.height() >= src.height() {
        (dest.height() / src.height()).max(1)
    } else {
        1
    };

    let span_w = tiles_x.saturating_mul(src.width());
    let span_h = tiles_y.saturating_mul(src.height());
    let end_col = dest
        .start()
        .col()
        .saturating_add(span_w.saturating_sub(1))
        .min(MAX_COL);
    let end_row = dest
        .start()
        .row()
        .saturating_add(span_h.saturating_sub(1))
        .min(MAX_ROW);
    let Some(end_addr) = CellAddr::new(end_col, end_row) else {
        return CellDelta::default();
    };
    let target_rect = CellRange::new(dest.start(), end_addr);

    let mut planned: BTreeMap<CellAddr, Option<CellContent>> = BTreeMap::new();
    calc.sheet().for_each_populated_in(target_rect, |addr, _| {
        planned.insert(addr, None);
    });

    if captured.is_empty() {
        return build_delta(calc, planned);
    }

    // Bound hostile huge-tile copies so a multi-million-cell paste cannot hang or OOM.
    let total_writes = u64::from(tiles_x)
        .saturating_mul(u64::from(tiles_y))
        .saturating_mul(u64::try_from(captured.len()).unwrap_or(u64::MAX));
    if total_writes > 250_000 {
        return CellDelta::default();
    }

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let base_c = dest
                .start()
                .col()
                .saturating_add(tx.saturating_mul(src.width()));
            let base_r = dest
                .start()
                .row()
                .saturating_add(ty.saturating_mul(src.height()));
            let dc = i64::from(base_c) - i64::from(src.start().col());
            let dr = i64::from(base_r) - i64::from(src.start().row());
            for (&(oc, or), content) in &captured {
                let Some(target) =
                    CellAddr::new(base_c.saturating_add(oc), base_r.saturating_add(or))
                else {
                    continue;
                };
                let shifted = match content {
                    CellContent::Formula(f) => CellContent::Formula(f.shifted(dc, dr)),
                    other => other.clone(),
                };
                planned.insert(target, Some(shifted));
            }
        }
    }

    build_delta(calc, planned)
}

fn plan_move(calc: &CalcSheet, src: CellRange, dest: CellRange) -> CellDelta {
    let dc = i64::from(dest.start().col()) - i64::from(src.start().col());
    let dr = i64::from(dest.start().row()) - i64::from(src.start().row());
    if dc == 0 && dr == 0 {
        return CellDelta::default();
    }

    let Some(dest_end_col) = dest
        .start()
        .col()
        .checked_add(src.width().saturating_sub(1))
    else {
        return CellDelta::default();
    };
    let Some(dest_end_row) = dest
        .start()
        .row()
        .checked_add(src.height().saturating_sub(1))
    else {
        return CellDelta::default();
    };
    let Some(dest_end) = CellAddr::new(dest_end_col, dest_end_row) else {
        return CellDelta::default();
    };
    let dest_rect = CellRange::new(dest.start(), dest_end);

    let mut captured: Vec<(CellAddr, CellContent)> = Vec::new();
    let mut planned: BTreeMap<CellAddr, Option<CellContent>> = BTreeMap::new();
    calc.sheet().for_each_populated_in(src, |addr, cell| {
        captured.push((addr, cell.content.clone()));
        planned.insert(addr, None);
    });
    calc.sheet().for_each_populated_in(dest_rect, |addr, _| {
        planned.insert(addr, None);
    });

    for (old_addr, content) in captured {
        let Some(new_addr) = old_addr.offset(dc, dr) else {
            continue;
        };
        let next = match content {
            CellContent::Formula(f) => CellContent::Formula(f.remap_move(src, dc, dr, dest_rect)),
            other => other,
        };
        planned.insert(new_addr, Some(next));
    }

    let mut dependents: HashSet<CellAddr> =
        calc.graph().dependents_of_range(src).into_iter().collect();
    dependents.extend(calc.graph().dependents_of_range(dest_rect));
    for dep in dependents {
        if src.contains(dep) || dest_rect.contains(dep) {
            continue;
        }
        if let Some(CellContent::Formula(f)) = calc.content(dep) {
            let rewritten = f.remap_move(src, dc, dr, dest_rect);
            if rewritten != *f {
                planned.insert(dep, Some(CellContent::Formula(rewritten)));
            }
        }
    }

    build_delta(calc, planned)
}

fn build_delta(calc: &CalcSheet, planned: BTreeMap<CellAddr, Option<CellContent>>) -> CellDelta {
    let mut changes = Vec::new();
    for (addr, after) in planned {
        let before = calc.content(addr).cloned();
        if before != after {
            changes.push((addr, before, after));
        }
    }
    CellDelta { changes }
}
