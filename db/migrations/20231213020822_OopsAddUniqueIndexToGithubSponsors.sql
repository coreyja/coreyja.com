-- Add migration script here
TRUNCATE TABLE GithubSponsors;

CREATE UNIQUE INDEX GithubSponsors_github_id ON GithubSponsors (github_id);
