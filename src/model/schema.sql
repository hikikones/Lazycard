CREATE TABLE topics (
  id INTEGER,
  name VARCHAR(255) NOT NULL,
  image VARCHAR(255),
  sort_order VARCHAR(255),
  parent_id INTEGER,
  PRIMARY KEY (id),
  FOREIGN KEY (parent_id) REFERENCES topics (id)
    ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE cards (
  id INTEGER,
  front TEXT,
  back TEXT,
  due_date DATE DEFAULT (date('now')) NOT NULL,
  due_days INT DEFAULT 0 NOT NULL,
  last_review_date DATE,
  topic_id INT NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (topic_id) REFERENCES topics (id)
    ON UPDATE CASCADE ON DELETE CASCADE
);

INSERT INTO topics (name, image, sort_order, parent_id)
  VALUES  ("Computer Science", NULL, NULL, NULL),
          ("Mathematics", NULL, NULL, NULL),
          ("Physics", NULL, NULL, NULL),
          ("Algorithms", NULL, NULL, 1),
          ("Sorting", NULL, NULL, 4),
          ("Data Structures", NULL, NULL, 4),
          ("Graphs", NULL, NULL, 4),
          ("Hash Tables", NULL, NULL, 6),
          ("Databases", NULL, NULL, 1),
          ("Normalization", NULL, NULL, 9),
          ("SQL", NULL, NULL, 9),
          ("Networking", NULL, NULL, 1),
          ("Big-O", NULL, NULL, 4),
          ("Advanced", NULL, NULL, 8),
          ("Test", NULL, NULL, NULL),
          ("Test", NULL, "3", 15),
          ("Test", NULL, "2", 15),
          ("Test", NULL, "1", 15),
          ("Test", NULL, "3.4", 16),
          ("Test", NULL, "3.3", 16),
          ("Test", NULL, "3.2", 16),
          ("Test", NULL, "3.1", 16),
          ("Test", NULL, "1.3", 18),
          ("Test", NULL, "1.4", 18),
          ("Test", NULL, "2.1", 17),
          ("Test", NULL, "2.4", 17),
          ("Test", NULL, "3.4.3", 19),
          ("Test", NULL, "3.4.1", 19),
          ("Test", NULL, "3.4.2", 19);

INSERT INTO cards (front, back, topic_id)
  VALUES  ("What does __SQL__ stand for?", "Structured Query Language", 11),
          ("What is database _normalization_", "The process of structuring a relational database with the goal to reduce data redundancy and improve data integrity.", 10),
          ("What is meant by data redundancy in a database system", "Some information is stored repeatedly.", 10),
          ("What is a _functional dependency_ (FD)?", "A FD is a constraint between two sets of attributes in a relation from a database.", 10),
          ("What is an _algorithm_?", "A step by step method of solving a problem.", 4),
          ("Worst time complexity for bubble sort?", "$$ O(n^2). $$", 4);


/*
--------------------------
RECURSIVE SUBTOPICS SEARCH
--------------------------

WITH RECURSIVE child_topics (topic_id, topic_name, parent_topic_id) AS (
  SELECT topic_id, topic_name, parent_topic_id
  FROM topics
  WHERE parent_topic_id = 1
  UNION ALL
  SELECT t.topic_id, t.topic_name, t.parent_topic_id
  FROM topics t
  INNER JOIN child_topics ON t.parent_topic_id = child_topics.topic_id
)
SELECT * FROM child_topics;

*/