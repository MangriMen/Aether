pub use inventory;

#[derive(Debug)]
pub struct SchemaEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub schema: fn() -> schemars::Schema,
}

inventory::collect!(SchemaEntry);

pub fn get_all_schemas() -> Vec<&'static SchemaEntry> {
    inventory::iter::<SchemaEntry>.into_iter().collect()
}
