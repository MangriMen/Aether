CREATE TABLE IF NOT EXISTS launcher_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  launcher_dir TEXT NOT NULL,
  metadata_dir TEXT NOT NULL,
  max_concurrent_downloads INTEGER NOT NULL DEFAULT 10
);

CREATE TABLE IF NOT EXISTS enabled_plugins (plugin_id TEXT PRIMARY KEY NOT NULL);