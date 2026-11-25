use proto123_engine::menu::verify;
use proto123_engine::{
    CATALOG, CommandId, CommandSpec, MenuKind, MenuNode, REGISTRY, StringKey, text,
};

#[test]
fn shipped_catalog_passes_and_matches_contract_tree() {
    let violations = verify(CATALOG, REGISTRY);
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );

    let tree: Vec<(&str, Vec<(&str, CommandId)>)> = CATALOG
        .iter()
        .map(|root| {
            let children = match root.kind {
                MenuKind::Branch(items) => items
                    .iter()
                    .map(|leaf| match leaf.kind {
                        MenuKind::Leaf(id) => (text(leaf.label), id),
                        MenuKind::Branch(_) => panic!("expected leaf"),
                    })
                    .collect(),
                MenuKind::Leaf(_) => panic!("expected branch"),
            };
            (text(root.label), children)
        })
        .collect();

    assert_eq!(
        tree,
        vec![
            ("File", vec![("New", CommandId::FileNewWorkbook)]),
            (
                "Edit",
                vec![("Undo", CommandId::EditUndo), ("Redo", CommandId::EditRedo),],
            ),
            (
                "Range",
                vec![
                    ("Erase", CommandId::RangeErase),
                    ("Copy", CommandId::RangeCopy),
                    ("Move", CommandId::RangeMove),
                ],
            ),
            ("View", vec![("Theme", CommandId::ViewCycleTheme)]),
        ]
    );
}

#[test]
fn seeded_bad_catalogs_report_the_menu_path() {
    // 1. Unregistered leaf id.
    let bad_leaf = [MenuNode {
        label: StringKey::MenuFile,
        desc: StringKey::MenuFileDesc,
        trigger: 'F',
        kind: MenuKind::Branch(&[MenuNode {
            label: StringKey::MenuFileNew,
            desc: StringKey::MenuFileNewDesc,
            trigger: 'N',
            kind: MenuKind::Leaf(CommandId::FileNewWorkbook),
        }]),
    }];
    let empty_registry: &[CommandSpec] = &[];
    let v = verify(&bad_leaf, empty_registry);
    assert!(
        v.iter()
            .any(|x| x.path == "/File/New" && x.message.contains("unregistered")),
        "{v:?}"
    );

    // 2. Duplicate trigger at the same level.
    let dup_trigger = [
        MenuNode {
            label: StringKey::MenuFile,
            desc: StringKey::MenuFileDesc,
            trigger: 'F',
            kind: MenuKind::Leaf(CommandId::FileNewWorkbook),
        },
        MenuNode {
            label: StringKey::MenuEdit,
            desc: StringKey::MenuEditDesc,
            trigger: 'F',
            kind: MenuKind::Leaf(CommandId::EditUndo),
        },
    ];
    let v = verify(&dup_trigger, REGISTRY);
    assert!(
        v.iter()
            .any(|x| x.path == "/Edit" && x.message.contains("duplicate trigger")),
        "{v:?}"
    );

    // 3. Trigger not equal to label's first letter.
    let bad_trigger = [MenuNode {
        label: StringKey::MenuView,
        desc: StringKey::MenuViewDesc,
        trigger: 'X',
        kind: MenuKind::Leaf(CommandId::ViewCycleTheme),
    }];
    let v = verify(&bad_trigger, REGISTRY);
    assert!(
        v.iter()
            .any(|x| x.path == "/View" && x.message.contains("first letter")),
        "{v:?}"
    );
}
