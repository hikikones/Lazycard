CREATE TABLE cards (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    create_time INTEGER DEFAULT (unixepoch('now')) NOT NULL,
    update_time INTEGER DEFAULT (unixepoch('now')) NOT NULL,

    -- Review state
    stability REAL DEFAULT 0.0 NOT NULL,
    difficulty REAL DEFAULT 0.0 NOT NULL,
    due_time INTEGER DEFAULT (unixepoch('now')) NOT NULL
);

CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    time INTEGER DEFAULT (unixepoch('now')) NOT NULL,
    success INTEGER NOT NULL,
    card_id INTEGER NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE card_tags (
    card_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, tag_id)
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
    FOREIGN KEY (tag_id) REFERENCES tags (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE VIRTUAL TABLE cards_fts USING fts5 (
    content,
    content='cards',
    content_rowid='id',
    tokenize='trigram'
);

-- Triggers for full text search
CREATE TRIGGER cards_fts_insert
AFTER INSERT ON cards
BEGIN
    INSERT INTO cards_fts(rowid, content)
    VALUES (new.id, new.content);
END;

CREATE TRIGGER cards_fts_delete
AFTER DELETE ON cards
BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, content)
    VALUES ('delete', old.id, old.content);
END;

CREATE TRIGGER cards_fts_update
AFTER UPDATE ON cards
BEGIN
    INSERT INTO cards_fts(cards_fts, rowid, content)
    VALUES ('delete', old.id, old.content);
    INSERT INTO cards_fts(rowid, content)
    VALUES (new.id, new.content);
END;

-- INSERT INTO tags (name)
--     VALUES  ('tag1'),
--             ('tag2'),
--             ('tag3'),
--             ('tag4');

-- INSERT INTO cards (content)
--     VALUES  ('single'),
--             ('front' || char(10) || char(10) || '---' || char(10) || char(10) || 'back'),
--             ('first' || char(10) || char(10) || '---' || char(10) || char(10) || 'second' || char(10) || char(10) || '---' || char(10) || char(10) || 'third'),
--             ('tagless card');

-- INSERT INTO card_tags (card_id, tag_id)
--     VALUES  (1, 1),
--             (2, 2),
--             (2, 3),
--             (3, 3);

-- INSERT INTO reviews (time, success, card_id)
--     VALUES  ((unixepoch('now','+0 day')), 0, 1),
--             ((unixepoch('now','+1 day')), 1, 1),
--             ((unixepoch('now','+2 day')), 0, 1);