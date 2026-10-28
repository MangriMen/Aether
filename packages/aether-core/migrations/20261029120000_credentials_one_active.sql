-- At most one active account. Older builds could leave several rows active (activating the target
-- before deactivating the rest), so keep the first one by insertion order and deactivate the others
-- before the index can be created.
UPDATE credentials
SET is_active = 0
WHERE is_active = 1
  AND rowid <> (SELECT MIN(rowid) FROM credentials WHERE is_active = 1);

CREATE UNIQUE INDEX IF NOT EXISTS credentials_one_active
  ON credentials (is_active)
  WHERE is_active = 1;
