
//! Spreadsheet utilities for tab or comma delimited files

extern crate core;
use std::io::*;
use std::fs::File;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
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
    pub headers: Vec<Cell>,
    /// Number of rows, not counting header
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Matrix of cells representing the spreadsheet data
    pub data: Vec<Vec<Cell>>,
}

impl core::ops::Index<String> for Spreadsheet {
    type Output = Vec<Cell>;
    fn index(&self, index: String) -> &Vec<Cell> {
        let m = Cell::String(index);
        for row in self.data.iter() {
            if let Some(_) = row.iter().find(|&cell| *cell == m) {
                return row;
            }
        }
        panic!("Value not found in spreadsheet");
    }
}

impl core::ops::Index<usize> for Spreadsheet {
    type Output = Vec<Cell>;
    fn index(&self, index: usize) -> &Vec<Cell> {
        self.data.get(index).unwrap()
    }
}

impl core::ops::IndexMut<usize> for Spreadsheet {
    fn index_mut(&mut self, index: usize) -> &mut Vec<Cell> {
        self.data.get_mut(index).unwrap()
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
    /// let s = Spreadsheet::read("table.tsv", '\t').unwrap();
    /// ```
    pub fn read(filename: &str, delimiter: char) -> Result<Spreadsheet> {
        let mut data: Vec<Vec<Cell>> = Vec::new();
        let mut headers: Vec<Cell> = Vec::new();
        let mut contents = BufReader::new(File::open(filename)?).lines();

        if let Some(Ok(header_line)) = contents.next() {
            for cell in header_line.split(delimiter) {
                if let Ok(x) = cell.parse::<i32>() {
                    headers.push(Cell::Integer(x));
                }
                else if let Ok(x) = cell.parse::<f32>() {
                    headers.push(Cell::Float(x));
                }
                else if let Ok(x) = cell.parse::<String>() {
                    headers.push(Cell::String(x));
                }
                else {
                    return Err(Error::from(ErrorKind::UnexpectedEof));
                }

            }
        }

        let cols = headers.len();
        while let Some(Ok(cells)) = contents.next() {
            let mut new_line: Vec<Cell> = Vec::new();
            for cell in cells.split('\t') {
                if let Ok(x) = cell.parse::<i32>() {
                    new_line.push(Cell::Integer(x));
                }
                else if let Ok(x) = cell.parse::<f32>() {
                    new_line.push(Cell::Float(x));
                }
                else if let Ok(x) = cell.parse::<String>() {
                    new_line.push(Cell::String(x));
                }
                else {
                    return Err(Error::from(ErrorKind::UnexpectedEof));
                }

            }
            if new_line.len() != headers.len() {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }
            data.push(new_line);      
        }

        Ok(Spreadsheet {
            rows: data.len(),
            headers,
            data,     
            cols,
        })
    }  

    /// Write the spreadsheet to a tab-separated file, consuming the 
    /// Spreadsheet in the process
    pub fn write(self, filename: &str) -> Result<()> {
        let mut writer = BufWriter::new(File::create(filename)?);

        // write the headers first
        for cell in self.headers {
            writeln!(writer, "{}\t", 
                match cell {
                    Cell::Float(f) => f.to_string(),
                    Cell::Integer(x) => x.to_string(),
                    Cell::String(x) => x,
                    _ => "".into(),
                }
            )?;
        }

        // iter through each row in the spreadsheet
        for row in self.data {
            for cell in row {
                writeln!(writer, "{}\t", 
                    match cell {
                        Cell::Float(f) => f.to_string(),
                        Cell::Integer(x) => x.to_string(),
                        Cell::String(x) => x,
                        _ => "".into(),
                    }
                )?;
            }
        }
        // Return an empty unit Ok
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
