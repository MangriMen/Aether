CREATE TABLE IF NOT EXISTS default_instance_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  max_memory INTEGER NOT NULL DEFAULT 2048,
  force_fullscreen BOOLEAN NOT NULL,
  window_width INTEGER NOT NULL DEFAULT 960,
  window_height INTEGER NOT NULL DEFAULT 540,
  pre_launch_hook TEXT,
  wrapper_hook TEXT,
  post_exit_hook TEXT
);

CREATE TABLE IF NOT EXISTS launch_args (
  owner_id TEXT NOT NULL,
  arg TEXT NOT NULL,
  pos INTEGER NOT NULL,
  PRIMARY KEY (owner_id, pos)
);

CREATE TABLE IF NOT EXISTS env_vars (
  owner_id TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  PRIMARY KEY (owner_id, key)
);

CREATE INDEX IF NOT EXISTS idx_launch_args_owner ON launch_args(owner_id);

CREATE INDEX IF NOT EXISTS idx_env_vars_owner ON env_vars(owner_id);