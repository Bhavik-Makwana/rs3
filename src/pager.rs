use crate::table::{PAGE_SIZE, ROW_SIZE, TABLE_MAX_PAGES};
use log::info;
use std::{fs::File, os::unix::fs::FileExt};
pub struct Pager {
    pub file_descriptor: File,
    pub file_length: u64,
    pub num_pages: u64,
    pub pages: Vec<Vec<u8>>,
}

impl Pager {
    pub fn pager_open(filename: &str) -> Self {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();
        let file_length = file.metadata().unwrap().len();
        let num_pages = (file_length + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;

        // Initialize pager with empty pages
        let mut pager = Self {
            file_descriptor: file,
            file_length,
            num_pages,
            pages: vec![vec![0; PAGE_SIZE]; TABLE_MAX_PAGES],
        };

        // Load existing pages from file
        for page_num in 0..num_pages {
            let mut buffer = vec![0; PAGE_SIZE];
            match pager
                .file_descriptor
                .read_exact_at(&mut buffer, page_num * PAGE_SIZE as u64)
            {
                Ok(_) => {
                    pager.pages[page_num as usize] = buffer;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // If we hit EOF, that's okay - just use what we have
                    pager.pages[page_num as usize] = buffer;
                }
                Err(e) => {
                    panic!("Error reading page from file: {}", e);
                }
            }
        }

        pager
    }

    pub fn fetch_page(&mut self, page_num: u64) -> &Vec<u8> {
        if page_num > self.num_pages {
            panic!("Tried to fetch page number out of bounds. {}", page_num);
        }

        if self.pages[page_num as usize].iter().all(|&x| x == 0) {
            // cache miss, read from file
            info!("Cache miss for page {}", page_num);
            let mut buffer = vec![0; PAGE_SIZE];
            let mut num_pages = self.file_length / PAGE_SIZE as u64;

            if self.file_length % PAGE_SIZE as u64 != 0 {
                num_pages += 1;
            }

            if page_num <= num_pages {
                if let Err(e) = self
                    .file_descriptor
                    .read_at(&mut buffer, page_num * PAGE_SIZE as u64)
                {
                    panic!("Error reading page from file: {}", e);
                }
            }
            self.pages[page_num as usize] = buffer;
        }
        return &self.pages[page_num as usize];
    }

    pub fn flush(
        &mut self,
        page_num: u64,
        num_remaining_rows: Option<usize>,
    ) -> Result<(), std::io::Error> {
        info!(
            "Flushing page {} with {} remaining rows",
            page_num,
            num_remaining_rows.unwrap_or(0)
        );
        if let Some(num_remaining_rows) = num_remaining_rows {
            let offset: usize = num_remaining_rows * ROW_SIZE;
            let bytes_to_write = &self.pages[page_num as usize][..offset];
            info!(
                "Writing {:?} to offset {}",
                bytes_to_write,
                page_num * PAGE_SIZE as u64
            );
            if let Err(e) = self
                .file_descriptor
                .write_all_at(bytes_to_write, page_num * PAGE_SIZE as u64)
            {
                return Err(e);
            } else {
                info!("Flushed {} bytes to file", bytes_to_write.len());
                return Ok(());
            }
        } else {
            // Write the entire page
            if let Err(e) = self
                .file_descriptor
                .write_at(&self.pages[page_num as usize], page_num * PAGE_SIZE as u64)
            {
                return Err(e);
            }
            info!("Flushed entire page {} to file", page_num);
        }
        Ok(())
    }
}

impl Default for Pager {
    fn default() -> Self {
        Self::pager_open("test.db")
    }
}
