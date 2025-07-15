-- Add NOT NULL constraints to timestamp fields with defaults
ALTER TABLE threads 
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE stitches 
    ALTER COLUMN created_at SET NOT NULL;