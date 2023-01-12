CREATE TABLE config (
    id INTEGER PRIMARY KEY,
    database_path TEXT DEFAULT NULL
);

INSERT INTO config (id, database_path) VALUES (1, NULL);