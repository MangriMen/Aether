use sqlx::{SqlitePool, pool::PoolOptions};

use crate::{
    core::migrations::run_migrations,
    features::instance::{PackStorage, ProviderId, infra::SqlitePackStorage},
};

/// Rows `SqlitePackStorage` wrote while Modrinth was `core:modrinth#modrinth-content` must read
/// as `aether#modrinth`, and still be found when looked up by the new id.
const INSTANCE_ID: &str = "instance";
const CONTENT_PATH: &str = "mods/sodium.jar";
const LEGACY_MODRINTH: &str = "core:modrinth#modrinth-content";

async fn create_pool() -> SqlitePool {
    // No `instances` row backs the pack: these tests are about provider ids, not cascades.
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .foreign_keys(false);

    let pool = PoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create in-memory SQLite pool");

    run_migrations(&pool).await;

    pool
}

async fn insert_legacy_pack_file(pool: &SqlitePool) {
    sqlx::query("INSERT INTO packs (instance_id) VALUES (?)")
        .bind(INSTANCE_ID)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO pack_files (instance_id, content_path, file_name, hash, update_provider_id) \
         VALUES (?, ?, 'sodium.jar', 'abc', ?)",
    )
    .bind(INSTANCE_ID)
    .bind(CONTENT_PATH)
    .bind(LEGACY_MODRINTH)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO pack_file_updates (instance_id, content_path, provider_id, content_id, version_id) \
         VALUES (?, ?, ?, 'AANobbMI', 'v1')",
    )
    .bind(INSTANCE_ID)
    .bind(CONTENT_PATH)
    .bind(LEGACY_MODRINTH)
    .execute(pool)
    .await
    .unwrap();
}

fn aether_modrinth() -> ProviderId {
    "aether#modrinth".parse().unwrap()
}

#[tokio::test]
async fn should_read_legacy_modrinth_rows_as_aether_modrinth_when_the_table_holds_old_ids() {
    let pool = create_pool().await;
    insert_legacy_pack_file(&pool).await;
    let storage = SqlitePackStorage::new(pool);

    let pack_file = storage
        .get_pack_file(INSTANCE_ID, CONTENT_PATH)
        .await
        .unwrap();

    assert_eq!(pack_file.update_provider_id, Some(aether_modrinth()));
    let update = pack_file.update.unwrap();
    assert_eq!(update.len(), 1);
    assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
}

#[tokio::test]
async fn should_find_legacy_modrinth_rows_when_searching_by_aether_modrinth() {
    let pool = create_pool().await;
    insert_legacy_pack_file(&pool).await;
    let storage = SqlitePackStorage::new(pool);

    let found = storage
        .find_by_provider_id(INSTANCE_ID, &aether_modrinth(), "AANobbMI")
        .await
        .unwrap();

    assert_eq!(found.map(|f| f.file_name).as_deref(), Some("sodium.jar"));
}

#[tokio::test]
async fn should_store_aether_modrinth_when_a_legacy_row_is_saved_again() {
    let pool = create_pool().await;
    insert_legacy_pack_file(&pool).await;
    let storage = SqlitePackStorage::new(pool.clone());

    let pack_file = storage
        .get_pack_file(INSTANCE_ID, CONTENT_PATH)
        .await
        .unwrap();
    storage
        .update_pack_file(INSTANCE_ID, CONTENT_PATH, &pack_file)
        .await
        .unwrap();

    let stored: Vec<String> = sqlx::query_scalar(
        "SELECT provider_id FROM pack_file_updates WHERE instance_id = ? \
         UNION ALL SELECT update_provider_id FROM pack_files WHERE instance_id = ?",
    )
    .bind(INSTANCE_ID)
    .bind(INSTANCE_ID)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(stored, ["aether#modrinth", "aether#modrinth"]);
}
