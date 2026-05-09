CREATE TABLE IF NOT EXISTS universal_cache (
  namespace TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  expires_at DATETIME NOT NULL,
  PRIMARY KEY (namespace, key)
);

CREATE INDEX IF NOT EXISTS idx_cache_expires ON universal_cache(expires_at);