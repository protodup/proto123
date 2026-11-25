use proto123_calc::CalcSheet;
use proto123_coord::{CellAddr, CellRange, RefAddr};
use proto123_formula::Formula;
use proto123_model::{Alignment, CellContent};
use proto123_ops::{CellDelta, Op, UndoStack};

fn addr(s: &str) -> CellAddr {
    RefAddr::parse(s).unwrap().addr
}

#[test]
fn cell_delta_invert_round_trip() {
    let delta = CellDelta {
        changes: vec![
            (addr("A1"), None, Some(CellContent::Number(10.0))),
            (
                addr("B2"),
                Some(CellContent::Label {
                    text: "hi".to_owned(),
                    align: Alignment::Left,
                    prefix: None,
                }),
                None,
            ),
        ],
    };
    assert_eq!(delta.invert().invert(), delta);
}

#[test]
fn new_command_after_undo_clears_redo_and_capacity_drops_oldest() {
    let mut stack = UndoStack::with_capacity(3);
    let mut calc = CalcSheet::new();
    for i in 1..=4 {
        let op = Op::SetCell {
            addr: addr("A1"),
            content: Some(CellContent::Number(f64::from(i))),
        };
        let d = op.plan(&calc);
        calc.apply(&d.after_changes());
        stack.push(d);
    }
    assert_eq!(stack.undo_len(), 3);
    let u = stack.undo().unwrap();
    calc.apply(&u.before_changes());
    assert_eq!(stack.redo_len(), 1);

    let op = Op::SetCell {
        addr: addr("B1"),
        content: Some(CellContent::Number(99.0)),
    };
    let d = op.plan(&calc);
    calc.apply(&d.after_changes());
    stack.push(d);
    assert!(!stack.can_redo());
}

#[test]
fn hundred_random_mutating_ops_undo_and_redo_to_exact_snapshots() {
    let mut calc = CalcSheet::new();
    let mut stack = UndoStack::new();
    let mut snapshots = vec![calc.sheet().clone()];

    let mut seed: u64 = 0x1234_5678_9abc_def0;
    let mut next = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        seed >> 33
    };

    for step in 0..100 {
        let c1 = u32::try_from(next() % 4).unwrap();
        let r1 = u32::try_from(next() % 5).unwrap();
        let a = CellAddr::new(c1, r1).unwrap();
        let op = match step % 4 {
            0 => Op::SetCell {
                addr: a,
                content: Some(CellContent::Number(f64::from(step))),
            },
            1 => {
                let target = CellAddr::new(c1, r1.saturating_sub(1)).unwrap();
                Op::SetCell {
                    addr: a,
                    content: Some(CellContent::Formula(
                        Formula::parse(&format!("{target}+1")).unwrap(),
                    )),
                }
            }
            2 => Op::Copy {
                src: CellRange::new(a, a),
                dest: CellRange::new(
                    CellAddr::new((c1 + 1) % 4, r1).unwrap(),
                    CellAddr::new((c1 + 1) % 4, (r1 + 1) % 5).unwrap(),
                ),
            },
            _ => Op::Erase {
                range: CellRange::new(a, a),
            },
        };
        let delta = op.plan(&calc);
        if !delta.is_empty() {
            calc.apply(&delta.after_changes());
            stack.push(delta);
            snapshots.push(calc.sheet().clone());
        }
    }

    assert!(stack.undo_len() >= 50);
    let final_sheet = calc.sheet().clone();

    while let Some(delta) = stack.undo() {
        snapshots.pop();
        calc.apply(&delta.before_changes());
        assert_eq!(calc.sheet(), snapshots.last().unwrap());
    }
    assert!(calc.sheet().is_empty());

    while let Some(delta) = stack.redo() {
        calc.apply(&delta.after_changes());
    }
    assert_eq!(calc.sheet(), &final_sheet);
}
