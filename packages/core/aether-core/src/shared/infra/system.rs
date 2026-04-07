use sysinfo::System;

pub fn get_total_memory() -> u64 {
    System::new_all().total_memory()
}
