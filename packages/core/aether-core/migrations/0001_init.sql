-- 1. CORE SETTINGS
CREATE TABLE IF NOT EXISTS launcher_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  launcher_dir TEXT NOT NULL,
  metadata_dir TEXT NOT NULL,
  max_concurrent_downloads INTEGER NOT NULL DEFAULT 10
);
--
CREATE TABLE IF NOT EXISTS enabled_plugins (plugin_id TEXT PRIMARY KEY NOT NULL);
-- 2. DEFAULT LAUNCH SETTINGS
CREATE TABLE IF NOT EXISTS instance_launch_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  -- MemorySettings
  max_memory INTEGER NOT NULL DEFAULT 2048,
  -- WindowSize
  force_fullscreen BOOLEAN,
  window_width INTEGER NOT NULL DEFAULT 960,
  window_height INTEGER NOT NULL DEFAULT 540,
  -- Hooks
  pre_launch_hook TEXT,
  wrapper_hook TEXT,
  post_exit_hook TEXT
);
-- 3. GAME INSTANCES
CREATE TABLE IF NOT EXISTS instances (
  id TEXT PRIMARY KEY,
  -- UUID or unique slug
  name TEXT NOT NULL,
  icon_path TEXT,
  install_stage TEXT NOT NULL,
  -- Enum as string
  -- Minecraft Metadata
  game_version TEXT NOT NULL,
  loader TEXT NOT NULL,
  -- Enum as string
  loader_version TEXT,
  -- Runtime Overrides (NULL means use global default)
  java_path TEXT,
  max_memory INTEGER,
  force_fullscreen BOOLEAN,
  window_width INTEGER,
  window_height INTEGER,
  -- Telemetry and Metadata
  time_played_total INTEGER NOT NULL DEFAULT 0,
  time_played_recent INTEGER NOT NULL DEFAULT 0,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_played_at DATETIME,
  -- Instance-specific Hooks
  pre_launch_hook TEXT,
  wrapper_hook TEXT,
  post_exit_hook TEXT,
  -- Complex metadata (PackInfo as JSON string)
  pack_info_json TEXT
);
-- 4. SHARED COLLECTIONS
-- owner_id corresponds to 'default' or instance.id
CREATE TABLE IF NOT EXISTS launch_args (
  owner_id TEXT NOT NULL,
  arg TEXT NOT NULL,
  pos INTEGER NOT NULL,
  -- Maintains argument order
  PRIMARY KEY (owner_id, pos)
);
-- owner_id corresponds to 'default' or instance.id
CREATE TABLE IF NOT EXISTS env_vars (
  owner_id TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  PRIMARY KEY (owner_id, key)
);
-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_launch_args_owner ON launch_args(owner_id);
CREATE INDEX IF NOT EXISTS idx_env_vars_owner ON env_vars(owner_id);
CREATE INDEX IF NOT EXISTS idx_instances_updated ON instances(updated_at);