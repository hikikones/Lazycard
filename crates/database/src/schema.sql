CREATE TABLE cards (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    due_date TEXT DEFAULT (datetime('now')) NOT NULL,
    due_days INTEGER DEFAULT 0 NOT NULL,
    recall_attempts INTEGER DEFAULT 0 NOT NULL,
    successful_recalls INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE tags (
    name TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE card_tag (
    card_id INTEGER NOT NULL,
    tag_name TEXT NOT NULL,
    PRIMARY KEY (card_id, tag_name)
    FOREIGN KEY (card_id) REFERENCES cards (id)
        ON UPDATE CASCADE ON DELETE CASCADE
    FOREIGN KEY (tag_name) REFERENCES tags (name)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE media (
    seahash BLOB PRIMARY KEY NOT NULL,
    bytes BLOB NOT NULL,
    file_ext TEXT NOT NULL
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

INSERT INTO card_tag (card_id, tag_name)
    VALUES  (1, "tag1"),
            (2, "tag2"),
            (2, "tag3"),
            (3, "tag3");
