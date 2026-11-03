pub struct Pager {
    page_length: u32,
    pages: Vec<Page>,
}

impl Pager {
    pub fn new(page_length: u32) -> Pager {
        Pager {
            page_length,
            pages: vec![],
        }
    }

    pub fn add_page(&mut self) {
        self.pages.push(Page::new())
    }

    pub fn get_page(&self, page_index: usize) -> Option<&Page> {
        if self.pages.len() >= page_index {
            Some(&self.pages[page_index])
        } else {
            None
        }
    }

    pub fn remove_page(&mut self, page_index: usize) -> PageOperationResult {
        if self.pages.len() >= page_index {
            self.pages.remove(page_index);
            PageOperationResult::Success
        } else {
            PageOperationResult::Failure
        }
    }

    pub fn get_num_pages(&self) -> usize {
        self.pages.len()
    }
}

struct Page {
    row_offset: u16,
    rows: Vec<Row>,
}

impl Page {
    pub fn new() -> Page {
        Page {
            row_offset: 0,
            rows: vec![],
        }
    }

    pub fn remove_row(&mut self, row_index: usize) -> PageOperationResult {
        if self.rows.len() >= row_index {
            self.rows.remove(row_index);
            PageOperationResult::Success
        } else {
            PageOperationResult::Failure
        }
    }

    pub fn add_row(&mut self, row: Row) -> PageOperationResult {
        // NOTE: Hard coded limit as of phase 2
        if self.rows.len() < 45 {
            self.rows.push(row);
            PageOperationResult::Success
        } else {
            PageOperationResult::Failure
        }
    }

    pub fn get_row(&self, row_index: usize) -> Option<&Row> {
        if self.rows.len() >= row_index {
            Some(&self.rows[row_index])
        } else {
            None
        }
    }

    pub fn set_row_offset(&mut self, offset: u16) -> PageOperationResult {
        // NOTE: Hard coded limit as of phase 2
        if offset <= 45 {
            self.row_offset = offset;
            PageOperationResult::Success
        } else {
            PageOperationResult::Failure
        }
    }

    pub fn get_row_offset(&self) -> u16 {
        self.row_offset
    }

    pub fn increment_row_offset(&mut self) -> PageOperationResult {
        // NOTE: Hard coded limit as of phase 2
        if self.row_offset < 45 {
            self.row_offset += 1;
            PageOperationResult::Success
        } else {
            PageOperationResult::Failure
        }
    }
}

struct Row {
    id: u32,
    first_name: String,
    last_name: String,
    is_active: bool,
}

pub enum PageOperationResult {
    Success,
    Failure,
}
