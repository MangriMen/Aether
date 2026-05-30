use sqlx::SqlitePool;
use tauri::AppHandle;

use super::state;

pub async fn run_migrations<R: tauri::Runtime>(app_handle: AppHandle<R>, pool: SqlitePool) {
    sqlx::migrate!("./migrations_links")
        .run(&pool)
        .await
        .expect("Migration failed");

    state::migrate(app_handle, pool)
        .await
        .expect("Migration failed");
}
