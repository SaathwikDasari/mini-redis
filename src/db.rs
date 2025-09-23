use std::collection::Hashmap;
use std::sync::{Arc, Mutex};

pub type SharedDb = Arc<Mutex<Hashmap<String, String>>>;

#[derive(clone)]
pub struct Db {
    inner: SharedDb,
}

impl Db {
    pub fn new() -> Self {
        Db {
            inner: Arc::new(Mutex::new(Hashmap::new()))
        }
    }
    pub fn set(&self, key: String, value: String) {
        self.inner.lock().unwrap().insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.lock().unwrap().get(key).cloned()
    }
}