use spreadsheet;
use spreadsheet::*;

#[test]
fn rowiter() {
    let spread = Spreadsheet::read("test.csv", ',').unwrap();
    let rows = spread
        .iter_rows()
        .map(Vec::from)
        .collect::<Vec<Vec<Cell>>>();
    assert_eq!(rows.len(), spread.rows);
    assert_eq!(
        rows[0],
        vec![Cell::Integer(0), Cell::Integer(1), Cell::Integer(2)]
    );
    assert_eq!(
        rows[1],
        vec![Cell::Integer(3), Cell::Integer(4), Cell::Integer(5)]
    );
}

#[test]
fn coliter() {
    let spread = Spreadsheet::read("test.csv", ',').unwrap();
    let cols = spread
        .iter_cols()
        .map(|i| i.collect::<Vec<_>>())
        .collect::<Vec<Vec<&Cell>>>();
    assert_eq!(cols.len(), spread.cols);
    assert_eq!(
        cols[0],
        vec![
            &Cell::Integer(0),
            &Cell::Integer(3),
            &Cell::Integer(6),
            &Cell::Integer(9)
        ]
    );
    assert_eq!(
        cols[1],
        vec![
            &Cell::Integer(1),
            &Cell::Integer(4),
            &Cell::Integer(7),
            &Cell::Integer(10)
        ]
    );
}

#[test]
fn iter() {
    let spread = Spreadsheet::read("test.csv", ',').unwrap();

    let cells = spread.iter(1, Direction::Column).collect::<Vec<_>>();
    let expected = vec![
        &Cell::Integer(1),
        &Cell::Integer(4),
        &Cell::Integer(7),
        &Cell::Integer(10),
    ];
    assert_eq!(cells, expected);

    let cells = spread.iter(1, Direction::Row).collect::<Vec<_>>();
    let expected = vec![&Cell::Integer(3), &Cell::Integer(4), &Cell::Integer(5)];
    assert_eq!(cells, expected);
}

#[test]
fn index() {
    let spread = Spreadsheet::read("test.csv", ',').unwrap();
    assert_eq!(spread[SymbolicIndex::new(0, "x")], Cell::Integer(0));
}
