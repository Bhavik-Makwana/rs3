use std::{fs::File, os::unix::fs::FileExt};
use crate::table::{PAGE_SIZE, TABLE_MAX_PAGES};
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
        Self { file_descriptor: file, file_length, num_pages, pages: vec![vec![0; PAGE_SIZE]; TABLE_MAX_PAGES] }
    }

    pub fn fetch_page(&mut self, page_num: u64) -> &Vec<u8> {
        if page_num > self.num_pages {
            panic!("Tried to fetch page number out of bounds. {}", page_num);
        }

        if self.pages[page_num as usize].iter().all(|&x| x == 0) {
            // cache miss, read from file
            println!("Cache miss for page {}", page_num);
            let mut buffer = vec![0; PAGE_SIZE];
            let mut num_pages = self.file_length / PAGE_SIZE as u64;

            if self.file_length % PAGE_SIZE as u64 != 0 {
                num_pages += 1;
            }

            if page_num <= num_pages {
                if let Err(e) = self.file_descriptor.read_at(&mut buffer, page_num * PAGE_SIZE as u64) {
                    panic!("Error reading page from file: {}", e);
                }
                
            }
            self.pages[page_num as usize] = buffer;
        }
        return &self.pages[page_num as usize]
    }

    pub fn flush(&mut self, page_num: u64) -> Result<(), std::io::Error> {
        if let Some(first_zero) = self.pages[page_num as usize].iter().position(|&x| x == 0) {
            // Only write up to the first zero byte
            let bytes_to_write = &self.pages[page_num as usize][..first_zero];
            if let Err(e) = self.file_descriptor.write_at(bytes_to_write, page_num * bytes_to_write.len() as u64) {
                return Err(e);
            } else {
                println!("Flushed {} bytes to file", self.pages[page_num as usize].len());
                return Ok(());
            }
        } else {
            // Write the entire page
            if let Err(e) = self.file_descriptor.write_at(&self.pages[page_num as usize], page_num * PAGE_SIZE as u64) {
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Default for Pager {
    fn default() -> Self {
        Self::pager_open("test.db")
    }
}