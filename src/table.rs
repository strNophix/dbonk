use crate::{layout::*, pager::Pager};

pub struct Table {
    pub row_count: usize,
    pub pager: Pager,
}

impl Table {
    pub fn new(pager: Pager) -> Self {
        let row_count = pager.file_len() / ROW_SIZE;
        Self { row_count, pager }
    }

    pub fn row_slot(&self, index: usize) -> (usize, usize) {
        self.pager.row_location(index)
    }

    pub fn close(&mut self) {
        let total_pages = self.row_count / ROWS_PER_PAGE;

        for i in 0..total_pages {
            let page = self.pager.pages[i];
            if page.is_none() {
                continue;
            }

            self.pager.flush_page(i).unwrap();
        }

        let added_rows = self.row_count % ROWS_PER_PAGE;
        if added_rows > 0 {
            let page_num = total_pages;
            if self.pager.pages[page_num].is_some() {
                self.pager.flush_page(page_num).unwrap();
            }
        }
    }
}
