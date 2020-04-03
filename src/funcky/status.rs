use std::collections::HashMap;
use std::sync::RwLock;

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Status {
    Accepted,
    Compiling,
    Failed(String),
    Ready,
}

#[derive(Clone, Debug, Serialize)]
pub struct FuncktionEntry {
    pub status: Status,
}

pub struct StatusTracker {
    registrations: RwLock<HashMap<String, FuncktionEntry>>,
}

impl StatusTracker {
    pub fn new() -> StatusTracker {
        StatusTracker {
            registrations: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, fn_name: &str) {
        let mut stat_guard = self.registrations.write().unwrap(); // TODO: Handle.
        stat_guard.insert(
            String::from(fn_name),
            FuncktionEntry {
                status: Status::Accepted,
            },
        );
    }

    pub fn get_status(&self, fn_name: &str) -> Option<Status> {
        let stat_guard = self.registrations.read().unwrap(); // TODO: Handle.
        stat_guard.get(fn_name).cloned().map(|f| f.status)
    }

    pub fn update_status(&self, fn_name: &str, new_status: Status) {
        let mut stat_guard = self.registrations.write().unwrap(); // TODO: Handle.
        let stat = stat_guard.get_mut(fn_name).unwrap(); // TODO: Handle.
        match &new_status {
            Status::Ready => assert_eq!(stat.status, Status::Compiling),
            Status::Failed(_) => assert_eq!(stat.status, Status::Compiling),
            Status::Compiling => assert_eq!(stat.status, Status::Accepted),
            _ => panic!(), // TODO: Handle.
        }
        stat.status = new_status;
    }

    /// Used for reloading funcktions at server startup.
    pub fn new_with_status(&self, fn_name: &str, new_status: Status) {
        let mut stat_guard = self.registrations.write().unwrap(); // TODO: Handle.
        stat_guard.insert(String::from(fn_name), FuncktionEntry { status: new_status });
    }

    pub fn all(&self) -> HashMap<String, FuncktionEntry> {
        let mut hsh = HashMap::new();
        let mut stat_guard = self.registrations.read().unwrap(); // TODO: Handle.

        for (k, v) in stat_guard.iter() {
            hsh.insert(k.clone(), v.clone());
        }

        hsh
    }
}
