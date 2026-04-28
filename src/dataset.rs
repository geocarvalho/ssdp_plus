//! CSV-backed dataset representation and train-style positive/negative indexing.

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

use csv::ReaderBuilder;

/// Tabular data loaded from CSV: attribute matrix and train-style positive/negative row splits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    /// Column names from the first CSV row (header).
    pub attributes: Vec<String>,
    /// Data rows as string cells (aligned with `attributes` order).
    pub records: Vec<Vec<String>>,
    /// Name of the target / label column (must appear in `attributes`).
    pub target_attr: String,
    /// String value in the target column that denotes the **positive** class.
    pub target_value: String,
    /// Row indices whose target column equals [`Dataset::target_value`] (positive instances).
    pub pos: Vec<usize>,
    /// Row indices whose target column differs from [`Dataset::target_value`] (negative instances).
    pub neg: Vec<usize>,
}

/// Errors produced while opening or parsing CSV data for a [`Dataset`].
#[derive(Debug)]
pub enum DatasetError {
    /// Underlying OS or file error.
    Io(std::io::Error),
    /// CSV parsing failure.
    Csv(csv::Error),
    /// The requested `target_attr` name was not found in the header row.
    MissingTargetAttr(String),
    /// File had a header but no body rows.
    EmptyRecords,
}

impl fmt::Display for DatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetError::Io(e) => write!(f, "IO error: {e}"),
            DatasetError::Csv(e) => write!(f, "CSV error: {e}"),
            DatasetError::MissingTargetAttr(name) => {
                write!(f, "target attribute `{name}` not found in header")
            }
            DatasetError::EmptyRecords => write!(f, "no data rows after header"),
        }
    }
}

impl Error for DatasetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DatasetError::Io(e) => Some(e),
            DatasetError::Csv(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DatasetError {
    fn from(e: std::io::Error) -> Self {
        DatasetError::Io(e)
    }
}

impl From<csv::Error> for DatasetError {
    fn from(e: csv::Error) -> Self {
        DatasetError::Csv(e)
    }
}

impl Dataset {
    /// Reads CSV from any [`Read`] source: first row is the header; builds [`Dataset`] and runs [`Dataset::split_pos_neg`].
    ///
    /// Does **not** print a summary (see [`Dataset::load`] for file paths + summary).
    pub fn from_reader<R: Read>(
        reader: R,
        separator: char,
        target_attr: &str,
        target_value: &str,
    ) -> Result<Dataset, DatasetError> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(separator as u8)
            .has_headers(true)
            .from_reader(reader);

        let headers = rdr.headers()?.clone();
        let attributes: Vec<String> = headers.iter().map(String::from).collect();

        let mut records = Vec::new();
        for result in rdr.records() {
            let rec = result?;
            records.push(rec.iter().map(String::from).collect());
        }

        if records.is_empty() {
            return Err(DatasetError::EmptyRecords);
        }

        let mut dataset = Dataset {
            attributes,
            records,
            target_attr: target_attr.to_string(),
            target_value: target_value.to_string(),
            pos: Vec::new(),
            neg: Vec::new(),
        };

        dataset.split_pos_neg()?;
        Ok(dataset)
    }

    /// Opens `path` and parses a [`Dataset`], then prints a short summary to stdout.
    #[allow(dead_code)]
    pub fn load(
        path: &str,
        separator: char,
        target_attr: &str,
        target_value: &str,
    ) -> Result<Dataset, DatasetError> {
        let file = File::open(path)?;
        let dataset = Self::from_reader(file, separator, target_attr, target_value)?;
        dataset.print_summary();
        Ok(dataset)
    }

    /// Recomputes [`Dataset::pos`] and [`Dataset::neg`] from the target column.
    pub fn split_pos_neg(&mut self) -> Result<(), DatasetError> {
        self.pos.clear();
        self.neg.clear();

        let col = self
            .attributes
            .iter()
            .position(|a| a == &self.target_attr)
            .ok_or_else(|| DatasetError::MissingTargetAttr(self.target_attr.clone()))?;

        for (i, row) in self.records.iter().enumerate() {
            let cell = row.get(col).map(String::as_str).unwrap_or("");
            if cell == self.target_value.as_str() {
                self.pos.push(i);
            } else {
                self.neg.push(i);
            }
        }

        Ok(())
    }

    /// Prints instance / class counts to stdout (used by [`Dataset::load`]).
    #[allow(dead_code)]
    pub fn print_summary(&self) {
        println!(
            "Dataset summary: {} instances, {} attributes, {} positives ({}), {} negatives",
            self.records.len(),
            self.attributes.len(),
            self.pos.len(),
            self.target_value,
            self.neg.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_split_inline_csv() {
        let csv = "a1,a2,class\n\
                   x,y,p\n\
                   u,v,n\n\
                   1,2,p\n";

        let d = Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("load");

        assert_eq!(d.attributes, vec!["a1", "a2", "class"]);
        assert_eq!(d.records.len(), 3);
        assert_eq!(d.pos, vec![0, 2]);
        assert_eq!(d.neg, vec![1]);
        assert_eq!(d.target_attr, "class");
        assert_eq!(d.target_value, "p");
    }

    #[test]
    fn semicolon_separator() {
        let csv = "f1;y\n\
                   0;n\n\
                   1;p\n";
        let d = Dataset::from_reader(csv.as_bytes(), ';', "y", "p").unwrap();
        assert_eq!(d.pos, vec![1]);
        assert_eq!(d.neg, vec![0]);
    }
}
