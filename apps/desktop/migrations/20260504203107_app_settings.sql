CREATE TABLE IF NOT EXISTS app_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  action_on_instance_launch TEXT NOT NULL,
  is_actual_transparent BOOLEAN NOT NULL,
  transparent BOOLEAN NOT NULL,
  window_effect TEXT NOT NULL
);