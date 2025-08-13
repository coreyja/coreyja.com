-- Drop Linear thread metadata table and associated objects
DROP TRIGGER IF EXISTS update_linear_thread_metadata_updated_at ON linear_thread_metadata;
DROP FUNCTION IF EXISTS update_linear_thread_metadata_updated_at();
DROP TABLE IF EXISTS linear_thread_metadata;