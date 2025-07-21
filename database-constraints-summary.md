# Database Constraints and Indexes Summary

## Overview

This document summarizes the database-level constraints and indexes added to improve data integrity and query performance in the Agentic Threads system.

## Threads Table Constraints

### Business Logic Constraints
- **check_interactive_no_branching**: Interactive threads cannot have a branching_stitch_id (they're always top-level conversations)
- **check_no_self_parent**: Prevents circular references where a thread is its own parent

### Performance Indexes
- **idx_threads_type**: Index on thread_type for filtering by autonomous/interactive
- **idx_threads_type_status**: Composite index for common query pattern (e.g., "find all running autonomous threads")
- **idx_threads_created_at**: Descending index on created_at for time-based queries

## Stitches Table Constraints

### Data Integrity Constraints
- **check_llm_call_fields**: LLM call stitches must have both request and response
- **check_tool_call_fields**: Tool call stitches must have tool name and input
- **check_thread_result_fields**: Thread result stitches must reference a child thread
- **check_discord_message_fields**: Discord message stitches must have message data
- **check_no_self_reference**: Prevents stitches from referencing themselves

### Performance Indexes
- **idx_stitches_thread_created**: Composite index for retrieving stitches in chronological order
- **idx_stitches_child_thread**: Partial index for finding parent stitches of child threads

## Discord Metadata Constraints

### Data Format Validation
- **Discord ID format checks**: Ensures all Discord IDs are valid snowflakes (numeric strings)
  - discord_thread_id, channel_id, guild_id, last_message_id
- **check_webhook_url_format**: Validates webhook URLs match Discord's format
- **check_participants_is_array**: Ensures participants field is always a JSON array

### Performance Indexes
- **idx_discord_guild_created**: Composite index for guild-specific queries ordered by time

## Design Notes

### Cross-Table Validation
PostgreSQL check constraints cannot reference other tables, so cross-table validations (like ensuring only interactive threads have Discord metadata or that stitch chains stay within the same thread) must be enforced at the application level.

The foreign key from `discord_thread_metadata.thread_id` to `threads.thread_id` combined with the primary key constraint ensures each thread has at most one Discord metadata record.

## Benefits

1. **Data Integrity**: Prevents invalid data states at the database level
2. **Performance**: Strategic indexes improve common query patterns
3. **Self-Documenting**: Constraints serve as executable documentation of business rules
4. **Early Error Detection**: Invalid data is rejected before it can corrupt the system
5. **Consistency**: Ensures data relationships remain valid across tables

## Query Performance Improvements

The indexes specifically optimize these common queries:
- Finding threads by type and status
- Retrieving recent threads
- Loading stitches in chronological order
- Finding Discord threads by guild
- Locating parent stitches of child threads

## Validation Examples

The constraints prevent invalid states like:
- Interactive threads having branching stitches
- Tool calls without specifying which tool
- Self-referencing threads or stitches
- Invalid Discord snowflake IDs
- Invalid webhook URL formats
- Non-array participants field