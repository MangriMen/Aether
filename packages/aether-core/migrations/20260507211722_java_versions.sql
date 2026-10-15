CREATE TABLE IF NOT EXISTS java_versions (
  major_version INTEGER PRIMARY KEY NOT NULL,
  -- 8, 17, 21, etc.
  version TEXT NOT NULL,
  -- Full version string
  architecture TEXT NOT NULL,
  path TEXT NOT NULL
);