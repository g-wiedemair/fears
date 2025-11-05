const DB_CAPACITY: usize = 1_000;

pub struct DataBase {
    bmat: Vec<u32>,
}

impl DataBase {
    pub fn new() -> DataBase {
        DataBase {
            bmat: Vec::with_capacity(DB_CAPACITY),
        }
    }
}
