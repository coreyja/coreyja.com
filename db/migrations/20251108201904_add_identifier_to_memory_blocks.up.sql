-- Add identifier column with default value 'default'
ALTER TABLE memory_blocks ADD COLUMN identifier text NOT NULL DEFAULT 'default';

-- Drop existing unique index on block_type
DROP INDEX IF EXISTS memory_blocks_block_type_unique_idx;

-- Rename block_type column to type for consistency
ALTER TABLE memory_blocks RENAME COLUMN block_type TO type;

-- Drop the check constraint on block_type (no longer needed with flexible types)
ALTER TABLE memory_blocks DROP CONSTRAINT IF EXISTS memory_blocks_block_type_check;

-- Add unique constraint on (type, identifier) combination
CREATE UNIQUE INDEX memory_blocks_type_identifier_unique_idx ON memory_blocks (type, identifier);
