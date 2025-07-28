-- Add unique index on block_type to ensure only one record per block type
CREATE UNIQUE INDEX memory_blocks_block_type_unique_idx ON memory_blocks (block_type);