-- Content paths are keyed with `/` (`mods/a.jar`). Imports on Windows used to record `mods\a.jar`,
-- which the content listing never finds: the file showed as new and could not be imported again.
-- Where both spellings exist, the `/` one is the one the listing uses and is kept.
PRAGMA defer_foreign_keys = ON;

UPDATE OR IGNORE pack_files
SET content_path = REPLACE(content_path, char(92), '/')
WHERE instr(content_path, char(92)) > 0;

-- A file row still spelled with `\` lost to a `/` twin. Its update records describe that other
-- file, so they go with it instead of being moved onto the twin.
DELETE FROM pack_file_updates
WHERE instr(content_path, char(92)) > 0
  AND EXISTS (
    SELECT 1 FROM pack_files f
    WHERE f.instance_id = pack_file_updates.instance_id
      AND f.content_path = pack_file_updates.content_path
  );

-- What is left belongs to a file that was renamed above.
UPDATE OR IGNORE pack_file_updates
SET content_path = REPLACE(content_path, char(92), '/')
WHERE instr(content_path, char(92)) > 0;

DELETE FROM pack_file_updates WHERE instr(content_path, char(92)) > 0;
DELETE FROM pack_files WHERE instr(content_path, char(92)) > 0;
