CREATE TABLE cards (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL
);

CREATE TABLE schedules (
    id INTEGER PRIMARY KEY,
    due_date TEXT DEFAULT (datetime('now')) NOT NULL,
    due_days INTEGER DEFAULT 0 NOT NULL,
    card_id INTEGER NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
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
    tag_id TEXT NOT NULL,
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
    VALUES  ("tag1"),
            ("tag2"),
            ("tag3"),
            ("tag4");

INSERT INTO cards (content) VALUES
("single"),
("front

---

back"),
("first

---

second

---

third"),
("tagless card");

INSERT INTO card_tag (card_id, tag_id)
    VALUES  (1, 1),
            (2, 2),
            (2, 3),
            (3, 3);
