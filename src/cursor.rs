use crate::table::Table;

pub type Position = (usize, usize);

pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub row_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn at_table_start(table: &'a mut Table) -> Self {
        let end_of_table = table.row_count == 0;
        Self {
            table,
            row_num: 0,
            end_of_table,
        }
    }

    pub fn at_table_end(table: &'a mut Table) -> Self {
        let row_count = table.row_count;
        Self {
            table,
            row_num: row_count,
            end_of_table: true,
        }
    }

    pub fn position(&self) -> Position {
        self.table.row_slot(self.row_num)
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        if self.row_num >= self.table.row_count {
            self.end_of_table = true;
        }
    }
}
