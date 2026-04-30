CREATE TABLE IF NOT EXISTS instances (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  icon_path TEXT,
  install_stage TEXT NOT NULL,
  game_version TEXT NOT NULL,
  loader TEXT NOT NULL,
  loader_version TEXT,
  java_path TEXT,
  max_memory INTEGER,
  force_fullscreen BOOLEAN,
  window_width INTEGER,
  window_height INTEGER,
  time_played_total INTEGER NOT NULL DEFAULT 0,
  time_played_recent INTEGER NOT NULL DEFAULT 0,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_played_at DATETIME,
  pre_launch_hook TEXT,
  wrapper_hook TEXT,
  post_exit_hook TEXT,
  pack_info_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_instances_updated ON instances(updated_at);