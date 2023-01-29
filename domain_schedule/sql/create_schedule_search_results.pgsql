CREATE TABLE IF NOT EXISTS schedule_search_results(
    id          SERIAL PRIMARY KEY,
    remote_id   VARCHAR NOT NULL,
    name        VARCHAR NOT NULL UNIQUE,
    description VARCHAR NOT NULL,
    type        VARCHAR NOT NULL
);
