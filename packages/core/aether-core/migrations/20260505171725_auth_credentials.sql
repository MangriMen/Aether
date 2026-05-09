CREATE TABLE IF NOT EXISTS credentials (
  id TEXT PRIMARY KEY NOT NULL,
  username TEXT NOT NULL,
  account_type TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT 0,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  expires TEXT NOT NULL
);