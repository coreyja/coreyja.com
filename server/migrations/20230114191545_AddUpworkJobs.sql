-- Add migration script here
CREATE TABLE
  UpworkJobs (
    id INTEGER PRIMARY KEY NOT NULL,
    guid TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );
