CREATE TABLE IF NOT EXISTS default_instance_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1) DEFAULT 1,
  -- Runtime
  java_path TEXT NOT NULL DEFAULT '',
  launch_args_json TEXT NOT NULL DEFAULT '[]',
  env_vars_json TEXT NOT NULL DEFAULT '[]',
  -- Memory
  max_memory INTEGER NOT NULL DEFAULT 2048,
  -- Window
  force_fullscreen BOOLEAN NOT NULL DEFAULT 0,
  window_width INTEGER NOT NULL DEFAULT 960,
  window_height INTEGER NOT NULL DEFAULT 540,
  -- Hooks
  hook_pre_launch TEXT NOT NULL DEFAULT '',
  hook_wrapper TEXT NOT NULL DEFAULT '',
  hook_post_exit TEXT NOT NULL DEFAULT ''
);