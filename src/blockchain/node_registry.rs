use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

static NODE_IDS_IN_USE: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| {
    Mutex::new(HashSet::new())
});

pub fn register_id(id: u32) -> Result<(), String> {
    let mut set = NODE_IDS_IN_USE.lock().unwrap();
    if set.contains(&id) {
        Err(format!("ID {} já está em uso", id))
    } else {
        set.insert(id);
        Ok(())
    }
}

pub fn unregister_id(id: u32) {
    let mut set = NODE_IDS_IN_USE.lock().unwrap();
    set.remove(&id);
}
