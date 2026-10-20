-- Content paths are keyed with `/` (`mods/a.jar`). A `mods\a.jar` key is never found by the
-- content listing: the file shows as new and cannot be imported again.
-- Where both spellings exist, the `/` one is the one the listing uses and is kept.
PRAGMA defer_foreign_keys = ON;

UPDATE OR IGNORE pack_file_updates
SET content_path = REPLACE(content_path, char(92), '/')
WHERE instr(content_path, char(92)) > 0;

UPDATE OR IGNORE pack_files
SET content_path = REPLACE(content_path, char(92), '/')
WHERE instr(content_path, char(92)) > 0;

-- What lost to a `/` twin.
DELETE FROM pack_file_updates WHERE instr(content_path, char(92)) > 0;
DELETE FROM pack_files WHERE instr(content_path, char(92)) > 0;
