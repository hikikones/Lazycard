CREATE TABLE cards (
    card_id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    due_date TEXT DEFAULT (datetime('now')) NOT NULL,
    due_days INTEGER DEFAULT 0 NOT NULL,
    recall_attempts INTEGER DEFAULT 0 NOT NULL,
    successful_recalls INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE tags (
    tag_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE card_tag (
    card_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, tag_id)
    FOREIGN KEY (card_id) REFERENCES cards (card_id)
        ON UPDATE CASCADE ON DELETE CASCADE
    FOREIGN KEY (tag_id) REFERENCES tags (tag_id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

INSERT INTO tags (name)
    VALUES  ("Tag1"),
            ("Tag2"),
            ("Tag3"),
            ("Tag4");

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
