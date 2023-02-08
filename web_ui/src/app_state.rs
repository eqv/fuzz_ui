use fuzzing_state::{FuzzingState};

use std::sync::Arc;
use std::sync::RwLock;


#[derive(Clone)]
pub struct AppState {
    fuzz: Arc<RwLock<FuzzingState>>,
}

impl AppState {
    pub fn get(&self) -> std::sync::RwLockReadGuard<FuzzingState> {
        return self.fuzz.read().unwrap();
    }

    pub fn get_mut(&self) -> std::sync::RwLockWriteGuard<FuzzingState> {
        return self.fuzz.write().unwrap();
    }

    pub fn new(state: FuzzingState) -> Self{
        return Self {
            fuzz: Arc::new(RwLock::new(state)),
        };
    }
}



