-- The install plan that put a file into the instance: the provider that returned the
-- plan and the content it was resolved for. The next plan of the same origin may replace or
-- remove the file. NULL for a file found on disk or installed some other way: no plan removes it.
ALTER TABLE pack_files ADD COLUMN installed_by_provider_id TEXT;
ALTER TABLE pack_files ADD COLUMN installed_by_content_id TEXT;
