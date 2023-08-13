use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::FileExt,
    path::Path,
};

use crate::{cursor::Position, layout::*};
use anyhow::Result;
use thiserror::Error;

pub type Page = [u8; PAGE_SIZE];

#[derive(Error, Debug)]
enum PagerError {
    #[error("Page {0} is out of bounds.")]
    OutOfBounds(usize),
    #[error("Read 0 bytes.")]
    NoBytes,
}

pub struct Pager {
    file: File,
    pub pages: [Option<Page>; TABLE_MAX_PAGES],
}

impl Pager {
    pub fn new<T: AsRef<Path>>(file_path: T) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;

        Ok(Self {
            file,
            pages: [None; TABLE_MAX_PAGES],
        })
    }

    pub fn file_len(&self) -> usize {
        let metadata = &self.file.metadata().expect("failed to parse metadata");
        metadata.len() as usize
    }

    pub fn row_location(&self, row_num: usize) -> Position {
        let page_num = row_num / ROWS_PER_PAGE;
        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;
        (page_num, byte_offset)
    }

    pub fn page(&mut self, page_num: usize) -> Result<&mut Page> {
        if page_num > TABLE_MAX_PAGES {
            return Err(PagerError::OutOfBounds(page_num).into());
        }

        if self.pages[page_num].is_none() {
            let file_len = self.file_len();
            let mut num_pages = file_len / PAGE_SIZE;

            if file_len % PAGE_SIZE != 0 {
                num_pages += 1
            }

            if page_num <= num_pages {
                let mut page: Page = [0; PAGE_SIZE];
                let offset: u64 = (page_num * PAGE_SIZE).try_into()?;
                self.file.read_at(&mut page, offset)?;
                self.pages[page_num] = Some(page);
            }
        }

        Ok(self.pages[page_num].as_mut().unwrap())
    }

    pub fn flush_page(&mut self, page_num: usize) -> Result<()> {
        match self.pages[page_num] {
            Some(page) => {
                let offset: u64 = (page_num * PAGE_SIZE).try_into()?;
                self.file.write_all_at(&page, offset)?;
                Ok(())
            }
            None => Err(PagerError::NoBytes.into()),
        }
    }
}
