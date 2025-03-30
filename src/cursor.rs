use crate::table::Table;
pub struct Cursor {
    pub row_num: usize,
    pub end_of_table: bool,
    pub table_size: usize,
}

pub enum CursorLocation {
    Start,
    End,
}

impl PartialEq for CursorLocation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CursorLocation::Start, CursorLocation::Start) => true,
            _ => false,
        }
    }
}

impl Cursor {
    pub fn table_start(table_size: usize) -> Self {
        Self {
            row_num: 0,
            end_of_table: false,
            table_size,
        }
    }

    pub fn table_end(num_rows: usize, table_size: usize) -> Self {
        Self {
            row_num: num_rows,
            end_of_table: true,
            table_size,
        }
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        if self.row_num >= self.table_size {
            self.end_of_table = true;
        }
    }
}
