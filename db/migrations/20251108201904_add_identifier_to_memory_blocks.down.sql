-- Drop unique constraint on (type, identifier)
DROP INDEX IF EXISTS memory_blocks_type_identifier_unique_idx;

-- Rename type column back to block_type
ALTER TABLE memory_blocks RENAME COLUMN type TO block_type;

-- Re-add check constraint for block_type
ALTER TABLE memory_blocks ADD CONSTRAINT memory_blocks_block_type_check
    CHECK (block_type IN ('persona'));

-- Re-add unique index on block_type
CREATE UNIQUE INDEX memory_blocks_block_type_unique_idx ON memory_blocks (block_type);

-- Drop identifier column
ALTER TABLE memory_blocks DROP COLUMN identifier;
