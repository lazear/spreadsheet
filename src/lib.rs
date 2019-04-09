//! Spreadsheet utilities for tab or comma delimited files

use std::fs::File;
use std::io::*;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
/// Represents a single cell in a spreadsheet
///
/// Three data types are represented: text, floats, and integers
pub enum Cell {
    /// Text cell
    Text(String),
    /// Floating point number
    Float(f64),
    /// Integer value
    Integer(i64),
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

    delimiter: char,
}

/// Index into a [`Spreadsheet`] by row and column number
pub struct NumericIndex {
    row: usize,
    col: usize,
}

/// Index into a [`Spreadsheet`] by row and column name, which must match
/// one of the headers
///
/// # Panics
///
/// Will panic if `col` is not in the headers of the [`Spreadsheet`], or
/// if the [`Spreadsheet`] does not have headers
pub struct SymbolicIndex<'s> {
    row: usize,
    col: &'s str,
}

impl<'s> SymbolicIndex<'s> {
    /// Create a new [`SymbolicIndex`]
    pub fn new(row: usize, col: &'s str) -> SymbolicIndex<'s> {
        SymbolicIndex { row, col }
    }
}

impl<'a> std::ops::Index<NumericIndex> for &'a Spreadsheet {
    type Output = Cell;
    fn index(&self, index: NumericIndex) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<'a, 's> std::ops::Index<SymbolicIndex<'s>> for &'a Spreadsheet {
    type Output = Cell;
    fn index(&self, index: SymbolicIndex) -> &Self::Output {
        let c = self.headers.iter().position(|i| i == index.col).unwrap();
        &self.data[index.row * self.cols + c]
    }
}

impl std::ops::Index<NumericIndex> for Spreadsheet {
    type Output = Cell;
    fn index(&self, index: NumericIndex) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<'s> std::ops::Index<SymbolicIndex<'s>> for Spreadsheet {
    type Output = Cell;
    fn index(&self, index: SymbolicIndex) -> &Self::Output {
        let c = self.headers.iter().position(|i| i == index.col).unwrap();
        &self.data[index.row * self.cols + c]
    }
}

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
    /// use spreadsheet::Spreadsheet;
    ///
    /// let s = Spreadsheet::read("./test.csv", '\t').unwrap();
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
                if let Ok(x) = cell.parse::<i64>() {
                    new_line.push(Cell::Integer(x));
                } else if let Ok(x) = cell.parse::<f64>() {
                    new_line.push(Cell::Float(x));
                } else if let Ok(x) = cell.parse::<String>() {
                    new_line.push(Cell::Text(x));
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
            delimiter,
        })
    }

    pub fn iter_rows(&self) -> Row<'_> {
        Row {
            data: &self.data,
            cols: self.cols,
            pos: 0,
        }
    }

    pub fn iter_cols(&self) -> Column<'_> {
        Column {
            data: &self,
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

    pub fn column(&self, col: &str) -> Option<Iter<'_>> {
        let idx = self.headers.iter().position(|i| i == col)?;
        Some(self.iter(idx, Direction::Column))
    }
}

/// An immutable iterator over rows in the [`Spreadsheet`]
pub struct Row<'a> {
    data: &'a [Cell],
    cols: usize,
    pos: usize,
}

impl<'a> Iterator for Row<'a> {
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

/// Immutable iterator over the columns in a [`Spreadsheet`].
/// Each column is returned as an [`Iter`] over the elements in that column.
pub struct Column<'a> {
    data: &'a Spreadsheet,
    cols: usize,
    pos: usize,
}

impl<'a> Iterator for Column<'a> {
    type Item = Iter<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.cols {
            let iter = self.data.iter(self.pos, Direction::Column);
            self.pos += 1;
            Some(iter)
        } else {
            None
        }
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
    }
}

// Write the spreadsheet to a tab-separated file, consuming the
// Spreadsheet in the process
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
