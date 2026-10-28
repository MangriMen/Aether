-- At most one active account: keep the first active row by insertion order, deactivate the rest,
-- then forbid a second one.
UPDATE credentials
SET is_active = 0
WHERE is_active = 1
  AND rowid <> (SELECT MIN(rowid) FROM credentials WHERE is_active = 1);

CREATE UNIQUE INDEX IF NOT EXISTS credentials_one_active
  ON credentials (is_active)
  WHERE is_active = 1;
