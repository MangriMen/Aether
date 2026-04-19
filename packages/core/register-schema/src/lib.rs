pub use inventory;
pub use register_schema_derive::RegisterSchema;
pub use schemars;

#[derive(Debug)]
pub struct SchemaEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub schema: fn() -> schemars::Schema,
}

inventory::collect!(SchemaEntry);

#[must_use = "The schemas must be registered or displayed; ignoring them is likely a bug."]
pub fn get_all_schemas() -> Vec<&'static SchemaEntry> {
    inventory::iter::<SchemaEntry>.into_iter().collect()
}
