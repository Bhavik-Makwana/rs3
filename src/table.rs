use crate::pager::Pager;
use log::{error, info};
use std::fmt;
use std::mem;
// Define the Row struct first (if not already defined)
#[repr(C)]
pub struct Row {
    pub id: u32,
    pub username: [u8; 32], // Assuming these sizes based on common SQLite implementations
    pub email: [u8; 255],
}

impl Row {
    pub fn new() -> Self {
        Row {
            id: 0,
            username: [0; 32],
            email: [0; 255],
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let username = String::from_utf8_lossy(&self.username).to_string();
        let email = String::from_utf8_lossy(&self.email).to_string();
        write!(
            f,
            "Row {{ id: {}, username: {}, email: {} }}",
            self.id, username, email
        )
    }
}

// Constants for field sizes
const ID_SIZE: usize = mem::size_of::<u32>();
const USERNAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;

// Field offsets
pub const ID_OFFSET: usize = 0;
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;
pub struct Table {
    pub num_rows: usize,
    pub pager: Pager,
}

pub type TableOffset = (usize, usize);

impl Default for Table {
    fn default() -> Self {
        Table::db_open("test.rdb")
    }
}

impl Default for Row {
    fn default() -> Self {
        Row::new()
    }
}

impl Table {
    pub fn db_open(filename: &str) -> Self {
        let pager = Pager::pager_open(filename);
        let file_length = pager.file_length as usize;
        Table {
            pager: pager,
            num_rows: file_length / ROW_SIZE,
        }
    }

    pub fn db_close(&mut self) {
        let num_full_pages = self.num_rows / ROWS_PER_PAGE;
        for i in 0..num_full_pages {
            if self.pager.pages[i].iter().all(|&x| x == 0) {
                continue;
            }
            self.pager.flush(i as u64, None).unwrap();
            self.pager.pages[i] = vec![0; PAGE_SIZE];
        }

        // Handle partial page
        let num_remaining_rows = self.num_rows % ROWS_PER_PAGE;
        if num_remaining_rows > 0 {
            let page_num = num_full_pages;
            if !self.pager.pages[page_num].iter().all(|&x| x == 0) {
                self.pager
                    .flush(page_num as u64, Some(num_remaining_rows))
                    .unwrap();
                self.pager.pages[page_num] = vec![0; PAGE_SIZE];
            }
        }

        if let Err(e) = self.pager.file_descriptor.sync_data() {
            error!("Error closing database file: {}", e);
        }
    }

    pub fn row_slot(&mut self, row_num: usize) -> TableOffset {
        let page_num = row_num / ROWS_PER_PAGE;
        // let page = self.pages[page_num];

        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        (page_num, byte_offset)
    }

    pub fn serialize_row(&mut self, source: &Row, destination: (usize, usize)) {
        let id = source.id;
        info!(
            "Serialize Row \nDestination {:?}\nLength of pages: {}",
            destination,
            self.pager.pages.len()
        );

        self.pager.pages[destination.0]
            [destination.1 + ID_OFFSET..destination.1 + ID_OFFSET + ID_SIZE]
            .copy_from_slice(&id.to_le_bytes());
        self.pager.pages[destination.0]
            [destination.1 + USERNAME_OFFSET..destination.1 + USERNAME_OFFSET + USERNAME_SIZE]
            .copy_from_slice(&source.username);
        self.pager.pages[destination.0]
            [destination.1 + EMAIL_OFFSET..destination.1 + EMAIL_OFFSET + EMAIL_SIZE]
            .copy_from_slice(&source.email);
    }

    pub fn deserialize_row(&mut self, page: usize, offset: usize) -> Row {
        info!("Deserialize Row \nPage: {}, Offset: {}", page, offset);
        Row {
            id: u32::from_le_bytes(
                self.pager.pages[page][offset + ID_OFFSET..offset + ID_OFFSET + ID_SIZE]
                    .try_into()
                    .unwrap(),
            ),
            username: self.pager.pages[page]
                [offset + USERNAME_OFFSET..offset + USERNAME_OFFSET + USERNAME_SIZE]
                .try_into()
                .unwrap(),
            email: self.pager.pages[page]
                [offset + EMAIL_OFFSET..offset + EMAIL_OFFSET + EMAIL_SIZE]
                .try_into()
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_creation() {
        let row = Row::new();
        assert_eq!(row.id, 0);
        assert_eq!(row.username, [0; 32]);
        assert_eq!(row.email, [0; 255]);
    }

    #[test]
    fn test_row_display() {
        let mut row = Row::new();
        row.id = 1;
        row.username[..5].copy_from_slice(b"alice");
        row.email[..14].copy_from_slice(b"alice@test.com");

        let display_string = format!("{}", row);
        assert!(display_string.contains("id: 1"));
        assert!(display_string.contains("username: alice"));
        assert!(display_string.contains("email: alice@test.com"));
    }

    #[test]
    fn test_table_serialization_deserialization() {
        let mut table = Table::db_open("test.rdb");
        let mut row = Row::new();
        row.id = 42;
        row.username[..5].copy_from_slice(b"alice");
        row.email[..14].copy_from_slice(b"alice@test.com");

        // Test row_slot calculation
        let slot = table.row_slot(0);
        assert_eq!(slot.0, 0); // First page
        assert_eq!(slot.1, 0); // First position

        // Test serialization
        table.serialize_row(&row, slot);

        // Test deserialization
        let deserialized_row = table.deserialize_row(slot.0, slot.1);
        assert_eq!(deserialized_row.id, 42);
        assert_eq!(&deserialized_row.username[..5], b"alice");
        assert_eq!(&deserialized_row.email[..14], b"alice@test.com");
    }

    #[test]
    fn test_table_multiple_rows() {
        let mut table = Table::db_open("test.rdb");

        // Create and insert first row
        let mut row1 = Row::new();
        row1.id = 1;
        row1.username[..3].copy_from_slice(b"bob");
        row1.email[..12].copy_from_slice(b"bob@test.com");

        // Create and insert second row
        let mut row2 = Row::new();
        row2.id = 2;
        row2.username[..5].copy_from_slice(b"alice");
        row2.email[..14].copy_from_slice(b"alice@test.com");

        // Insert both rows
        let slot1 = table.row_slot(0);
        let slot2 = table.row_slot(1);
        table.serialize_row(&row1, slot1);
        table.serialize_row(&row2, slot2);

        // Verify both rows
        let retrieved_row1 = table.deserialize_row(slot1.0, slot1.1);
        let retrieved_row2 = table.deserialize_row(slot2.0, slot2.1);

        assert_eq!(retrieved_row1.id, 1);
        assert_eq!(&retrieved_row1.username[..3], b"bob");
        assert_eq!(&retrieved_row1.email[..12], b"bob@test.com");

        assert_eq!(retrieved_row2.id, 2);
        assert_eq!(&retrieved_row2.username[..5], b"alice");
        assert_eq!(&retrieved_row2.email[..14], b"alice@test.com");
    }

    #[test]
    fn test_table_page_boundary() {
        let mut table = Table::db_open("test.rdb");

        // Calculate how many rows we need to fill a page and spill over
        let rows_for_test = ROWS_PER_PAGE + 2;

        // Create and insert rows
        for i in 0..rows_for_test {
            let mut row = Row::new();
            row.id = i as u32;
            row.username[..4].copy_from_slice(b"user");
            row.email[..13].copy_from_slice(b"user@test.com");

            let slot = table.row_slot(table.num_rows);
            table.serialize_row(&row, slot);
            table.num_rows += 1;
            // Verify the page number calculation
            println!("slot: {:?}, i: {}", slot, i);
            assert_eq!(slot.0, i / ROWS_PER_PAGE);
            assert_eq!(slot.1, (i % ROWS_PER_PAGE) * ROW_SIZE);
        }

        // Verify rows from both pages
        for i in 0..rows_for_test {
            let slot = table.row_slot(i);
            let retrieved_row = table.deserialize_row(slot.0, slot.1);
            assert_eq!(retrieved_row.id, i as u32);
            assert_eq!(&retrieved_row.username[..4], b"user");
            assert_eq!(&retrieved_row.email[..13], b"user@test.com");
        }

        // Verify we actually used multiple pages
        assert!(table.pager.pages.len() > 1);
    }
}
