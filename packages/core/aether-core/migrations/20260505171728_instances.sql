CREATE TABLE IF NOT EXISTS instances (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  icon_path TEXT,
  install_stage TEXT NOT NULL,
  -- Minecraft Metadata
  game_version TEXT NOT NULL,
  loader TEXT NOT NULL,
  -- LoaderVersionPreference: Stable, Latest, Exact
  loader_version_json TEXT,
  -- Launch Settings
  java_path TEXT,
  -- Json strings
  launch_args_json TEXT,
  env_vars_json TEXT,
  -- MemorySettings (Flattened)
  memory_maximum INTEGER NOT NULL DEFAULT 2048,
  -- WindowSize & Display (Flattened)
  force_fullscreen BOOLEAN NOT NULL DEFAULT 0,
  window_width INTEGER NOT NULL DEFAULT 960,
  window_height INTEGER NOT NULL DEFAULT 540,
  -- Timestamps
  created_at DATETIME NOT NULL,
  modified_at DATETIME NOT NULL,
  last_played_at DATETIME,
  -- Stats
  time_played INTEGER NOT NULL DEFAULT 0,
  recent_time_played INTEGER NOT NULL DEFAULT 0,
  -- Hooks (Flattened)
  hook_pre_launch TEXT,
  hook_wrapper TEXT,
  hook_post_exit TEXT
);

CREATE TABLE IF NOT EXISTS instance_pack_info (
  instance_id TEXT PRIMARY KEY NOT NULL,
  provider_id TEXT NOT NULL,
  modpack_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_instances_modified ON instances(modified_at);