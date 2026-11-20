-- The games the launcher started, kept so that a later launcher finds the ones still running.
-- A process is told apart from a later one that reuses its pid by the moment it started, compared
-- exactly: on Linux the start in clock ticks since boot plus the boot id, elsewhere the creation
-- time in seconds with an empty boot id. The owner is the launcher that follows the game.
CREATE TABLE IF NOT EXISTS game_processes (
  id TEXT PRIMARY KEY NOT NULL,
  instance_id TEXT NOT NULL,
  pid INTEGER NOT NULL,
  start_stamp INTEGER NOT NULL,
  boot_id TEXT NOT NULL DEFAULT '',
  started_at DATETIME NOT NULL,
  -- 'process' | 'tree'
  kill_scope TEXT NOT NULL,
  post_exit_command TEXT NOT NULL DEFAULT '',
  playtime_credited_until DATETIME NOT NULL,
  owner_pid INTEGER NOT NULL,
  owner_start_stamp INTEGER NOT NULL,
  owner_boot_id TEXT NOT NULL DEFAULT '',
  FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_game_processes_instance ON game_processes(instance_id);
