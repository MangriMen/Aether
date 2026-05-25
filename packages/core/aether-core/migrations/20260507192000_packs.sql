CREATE TABLE IF NOT EXISTS packs (
  instance_id TEXT PRIMARY KEY,
  FOREIGN KEY(instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pack_files (
  instance_id TEXT NOT NULL,
  content_path TEXT NOT NULL,
  -- PackFile fields
  file_name TEXT NOT NULL,
  name TEXT,
  hash TEXT NOT NULL,
  side TEXT,
  update_provider_id TEXT,
  PRIMARY KEY(instance_id, content_path),
  FOREIGN KEY(instance_id) REFERENCES packs(instance_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pack_file_updates (
  instance_id TEXT NOT NULL,
  content_path TEXT NOT NULL,
  provider_id TEXT NOT NULL,
  content_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  PRIMARY KEY(instance_id, content_path, provider_id),
  FOREIGN KEY(instance_id, content_path) REFERENCES pack_files(instance_id, content_path) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_pack_file_updates_search ON pack_file_updates(instance_id, provider_id, content_id);