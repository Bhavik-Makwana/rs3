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
    pub pages: Vec<Vec<u8>>,
}

impl Default for Table {
    fn default() -> Self {
        Table::new()
    }
}

impl Default for Row {
    fn default() -> Self {
        Row::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Table {
            num_rows: 0,
            pages: vec![vec![0; PAGE_SIZE]; TABLE_MAX_PAGES],
        }
    }

    pub fn row_slot(&mut self, row_num: usize) -> (usize, usize) {
        let page_num = row_num / ROWS_PER_PAGE;
        // let page = self.pages[page_num];

        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        (page_num, byte_offset)
    }

    pub fn serialize_row(&mut self, source: &Row, destination: (usize, usize)) {
        let id = source.id;
        // println!("id: {}", id);
        println!("destination: {:?}", destination);
        // println!("Length of pages: {}", self.pages.len());
        self.pages[destination.0][destination.1 + ID_OFFSET..destination.1 + ID_OFFSET + ID_SIZE]
            .copy_from_slice(&id.to_le_bytes());
        self.pages[destination.0]
            [destination.1 + USERNAME_OFFSET..destination.1 + USERNAME_OFFSET + USERNAME_SIZE]
            .copy_from_slice(&source.username);
        self.pages[destination.0]
            [destination.1 + EMAIL_OFFSET..destination.1 + EMAIL_OFFSET + EMAIL_SIZE]
            .copy_from_slice(&source.email);
    }

    pub fn deserialize_row(&mut self, page: usize, offset: usize) -> Row {
        Row {
            id: u32::from_le_bytes(
                self.pages[page][offset + ID_OFFSET..offset + ID_OFFSET + ID_SIZE]
                    .try_into()
                    .unwrap(),
            ),
            username: self.pages[page]
                [offset + USERNAME_OFFSET..offset + USERNAME_OFFSET + USERNAME_SIZE]
                .try_into()
                .unwrap(),
            email: self.pages[page][offset + EMAIL_OFFSET..offset + EMAIL_OFFSET + EMAIL_SIZE]
                .try_into()
                .unwrap(),
        }
    }
}
