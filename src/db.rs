use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;

#[derive(Clone)]
pub struct Db {
    shared: Arc<Mutex<HashMap<String, Bytes>>>
}

impl Db {
    pub fn new() -> Db {
        Db {
            shared: Arc::new(Mutex::new(HashMap::new()))
        }

        
    }
    pub fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.lock().unwrap();

        state.get(key).cloned()
    }

    pub fn set(&self, key: String, value: Bytes) {
        let mut state = self.shared.lock().unwrap();
        state.insert(key, value);
    }

}