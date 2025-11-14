-- Remove SEC Reporting persona
DELETE FROM memory_blocks
WHERE type = 'persona' AND identifier = 'sec-reporting';
