CREATE TABLE memory_blocks (
    memory_block_id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
    block_type text NOT NULL,
    content text NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT now(),
    updated_at timestamp with time zone NOT NULL DEFAULT now()
);

-- Add check constraint for block types
ALTER TABLE memory_blocks ADD CONSTRAINT memory_blocks_block_type_check 
    CHECK (block_type IN ('persona'));