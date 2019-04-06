//! Spreadsheet utilities for tab or comma delimited files

extern crate core;
use std::fs::File;
use std::io::*;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
/// Represents a single cell in a spreadsheet
///
/// Three data types are represented: text, floats, and integers
pub enum Cell {
    /// Text cell
    String(String),
    /// Floating point number
    Float(f32),
    /// Integer value
    Integer(i32),
    /// No value
    Empty,
}

/// Spreadsheet struct
pub struct Spreadsheet {
    /// Vector of cells representing the first row
    /// in the spreadsheet
    pub headers: Vec<String>,
    /// Number of rows, not counting header
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Matrix of cells representing the spreadsheet data
    pub data: Vec<Cell>,
}

pub struct NumericIndex {
    row: usize,
    col: usize,
}

pub struct SymbolicIndex<'s> {
    row: usize,
    col: &'s str,
}

impl<'s> SymbolicIndex<'s> {
    pub fn new(row: usize, col: &'s str) -> SymbolicIndex<'s> {
        SymbolicIndex {
            row,
            col
        }
    }
}

impl<'a> core::ops::Index<NumericIndex> for &'a Spreadsheet {
    type Output = Cell;
    fn index(&self, index: NumericIndex) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<'a, 's> core::ops::Index<SymbolicIndex<'s>> for &'a Spreadsheet {
    type Output = Cell;
    fn index(&self, index: SymbolicIndex) -> &Self::Output {
        let c = self.headers.iter().position(|i| i == index.col).unwrap();
        &self.data[index.row * self.cols + c]
    }
}

impl core::ops::Index<NumericIndex> for Spreadsheet {
    type Output = Cell;
    fn index(&self, index: NumericIndex) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<'s> core::ops::Index<SymbolicIndex<'s>> for Spreadsheet {
    type Output = Cell;
    fn index(&self, index: SymbolicIndex) -> &Self::Output {
        let c = self.headers.iter().position(|i| i == index.col).unwrap();
        &self.data[index.row * self.cols + c]
    }
}

// impl<'a> core::ops::Index<String> for &'a Spreadsheet {
//     type Output = Iter<'a>;
//     fn index(&self, index: String) -> &Iter<'a> {
//         let idx = self.headers.iter().position(|i| i == &index).unwrap();
//         self.iter(idx, Direction::Column)
//     }
// }

// impl core::ops::Index<usize> for Spreadsheet {
//     type Output = Vec<Cell>;
//     fn index(&self, index: usize) -> &Vec<Cell> {
//         self.data.get(index).unwrap()
//     }
// }

// impl core::ops::IndexMut<usize> for Spreadsheet {
//     fn index_mut(&mut self, index: usize) -> &mut Vec<Cell> {
//         self.data.get_mut(index).unwrap()
//     }
// }

impl Spreadsheet {
    /// Read from a file into a struct Spreadsheet, returning a Result<Spreadsheet>
    ///
    /// # Arguments
    /// * `filename`: A string representing a file path
    /// * `delimiter`: A character to delimit cells, i.e. '\t' or ','
    ///
    /// # Example
    ///
    /// ```
    /// let s = Spreadsheet::read("table.tsv", '\t').unwrap();
    /// ```
    pub fn read(filename: &str, delimiter: char) -> Result<Spreadsheet> {
        let mut data: Vec<Cell> = Vec::new();
        let mut headers: Vec<String> = Vec::new();
        let mut contents = BufReader::new(File::open(filename)?).lines();

        if let Some(Ok(header_line)) = contents.next() {
            headers.extend(header_line.split(delimiter).map(String::from));
        }

        let cols = headers.len();
        let mut rows = 0;
        while let Some(Ok(cells)) = contents.next() {
            let mut new_line: Vec<Cell> = Vec::new();
            for cell in cells.split(delimiter) {
                if let Ok(x) = cell.parse::<i32>() {
                    new_line.push(Cell::Integer(x));
                } else if let Ok(x) = cell.parse::<f32>() {
                    new_line.push(Cell::Float(x));
                } else if let Ok(x) = cell.parse::<String>() {
                    new_line.push(Cell::String(x));
                } else {
                    return Err(Error::from(ErrorKind::UnexpectedEof));
                }
            }
            if new_line.len() != headers.len() {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }
            rows += 1;
            data.extend(new_line);
        }

        Ok(Spreadsheet {
            headers,
            data,
            cols,
            rows,
        })
    }

    pub fn iter_rows(&self) -> RowIter<'_> {
        RowIter {
            data: &self.data,
            cols: self.cols,
            pos: 0,
        }
    }

    pub fn iter_cols(&self) -> ColIter<'_> {
        ColIter {
            data: &self.data,
            rows: self.rows,
            cols: self.cols,
            pos: 0,
        }
    }

    pub fn iter(&self, pos: usize, dir: Direction) -> Iter<'_> {
        match dir {
            Direction::Column => Iter {
                dir,
                pos,
                data: &self.data,
                rows: self.rows,
                cols: self.cols,
            },
            Direction::Row => Iter {
                dir,
                pos: 0,
                data: &self.data,
                rows: pos,
                cols: self.cols,
            },
        }
    }
}

/// An immutable iterator over rows in the [`Spreadsheet`]
pub struct RowIter<'a> {
    data: &'a [Cell],
    cols: usize,
    pos: usize,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = &'a [Cell];
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos * self.cols;
        let end = start + self.cols;
        if end < self.data.len() + 1 {
            self.pos += 1;
            Some(&self.data[start..end])
        } else {
            None
        }
    }
}

/// Immutable iterator over the columns in a [`Spreadsheet`]
///
/// # Note
///
/// [`ColIter`] allocates additional heap memory
pub struct ColIter<'a> {
    data: &'a [Cell],
    cols: usize,
    rows: usize,
    pos: usize,
}

impl<'a> Iterator for ColIter<'a> {
    type Item = Vec<&'a Cell>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut vec = Vec::new();
        for r in 0..self.rows {
            vec.push(self.data.get(r * self.cols + self.pos)?);
        }
        self.pos += 1;
        Some(vec)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Direction {
    Column,
    Row,
}

/// Immutable iterator over [`Cell`]'s in a single row or column of a
/// [`Spreadsheet`], one cell at a  time.
#[derive(Copy, Clone)]
pub struct Iter<'a> {
    data: &'a [Cell],
    cols: usize,
    rows: usize,
    pos: usize,
    dir: Direction,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Cell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            return None;
        }
        match self.dir {
            Direction::Column => {
                let cell = self.data.get(self.pos);
                self.pos += self.cols;
                cell
            }
            Direction::Row => {
                if self.pos >= self.cols {
                    return None;
                }
                let cell = self.data.get(self.rows * self.cols + self.pos);
                self.pos += 1;
                cell
            }
        }
        // let r = match self.dir {
        //     Direction::Column => self.pos,
        //     Direction::Row => self.pos * self.cols;
        // }
        // dbg!(r);
        // if r < self.data.len() {
        //     let cell = self.data.get(r);
        //     self.pos += match self.dir {
        //         Direction::Column => self.cols,
        //         Direction::Row => 1,
        //     };
        //     cell
        // } else {
        //     None
        // }
    }
}

/// Write the spreadsheet to a tab-separated file, consuming the
/// Spreadsheet in the process
// pub fn write(self, filename: &str) -> Result<()> {
//     let mut writer = BufWriter::new(File::create(filename)?);

//     // write the headers first
//     for cell in self.headers {
//         writeln!(writer, "{}\t",
//             match cell {
//                 Cell::Float(f) => f.to_string(),
//                 Cell::Integer(x) => x.to_string(),
//                 Cell::String(x) => x,
//                 _ => "".into(),
//             }
//         )?;
//     }

//     // iter through each row in the spreadsheet
//     for row in self.data {
//         for cell in row {
//             writeln!(writer, "{}\t",
//                 match cell {
//                     Cell::Float(f) => f.to_string(),
//                     Cell::Integer(x) => x.to_string(),
//                     Cell::String(x) => x,
//                     _ => "".into(),
//                 }
//             )?;
//         }
//     }
//     // Return an empty unit Ok
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rowiter() {
        let spread = Spreadsheet::read("test.tsv", ',').unwrap();
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
        let spread = Spreadsheet::read("test.tsv", ',').unwrap();
        let cols = spread.iter_cols().collect::<Vec<Vec<&Cell>>>();
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
        let spread = Spreadsheet::read("test.tsv", ',').unwrap();

        let cells = spread.iter(1, Direction::Column).collect::<Vec<_>>();
        let expected = vec![
            &Cell::Integer(1),
            &Cell::Integer(4),
            &Cell::Integer(7),
            &Cell::Integer(10),
        ];
        assert_eq!(cells, expected);

        let cells = spread.iter(1, Direction::Row).collect::<Vec<_>>();
        let expected = vec![
            &Cell::Integer(3),
            &Cell::Integer(4),
            &Cell::Integer(5),
        ];
        assert_eq!(cells, expected);
    }

    #[test]
    fn index() {
        let spread = Spreadsheet::read("test.tsv", ',').unwrap();
        assert_eq!(spread[SymbolicIndex::new(0, "x")], Cell::Integer(0));

    }
}
