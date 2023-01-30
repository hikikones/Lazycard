CREATE TABLE cards (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    create_date TEXT DEFAULT (date('now')) NOT NULL,
    review_date TEXT DEFAULT (date('now')) NOT NULL
);

CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    date TEXT DEFAULT (date('now')) NOT NULL,
    success INTEGER NOT NULL,
    card_id INTEGER NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE card_tag (
    card_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, tag_id)
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
    FOREIGN KEY (tag_id) REFERENCES tags (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE assets (
    seahash BLOB PRIMARY KEY NOT NULL,
    bytes BLOB NOT NULL,
    extension TEXT NOT NULL
);

INSERT INTO tags (name)
    VALUES  ('tag1'),
            ('tag2'),
            ('tag3'),
            ('tag4');

INSERT INTO cards (content)
    VALUES  ('single'),
            ('front' || char(10) || '---' || char(10) || 'back'),
            ('first' || char(10) || '---' || char(10) || 'second' || char(10) || '---' || char(10) || 'third'),
            ('tagless card');

INSERT INTO card_tag (card_id, tag_id)
    VALUES  (1, 1),
            (2, 2),
            (2, 3),
            (3, 3);
