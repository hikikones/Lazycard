CREATE TABLE topics (
  id INTEGER,
  name TEXT NOT NULL,
  image TEXT,
  sort_order TEXT,
  parent_id INTEGER,
  PRIMARY KEY (id),
  FOREIGN KEY (parent_id) REFERENCES topics (id)
    ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE cards (
  id INTEGER,
  front_md TEXT,
  front_html TEXT,
  back_md TEXT,
  back_html TEXT,
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

INSERT INTO cards (front_md, front_html, back_md, back_html, topic_id)
  VALUES  ("What does SQL stand for?", "<p>What does SQL stand for?</p>", "Structured Query Language", "<p>Structured Query Language</p>", 11),
          ("What is database normalization", "<p>What is database normalization?</p>", "The process of structuring a relational database with the goal to reduce data redundancy and improve data integrity.", "<p>The process of structuring a relational database with the goal to reduce data redundancy and improve data integrity.</p>", 10),
          ("What is meant by data redundancy in a database system", "<p>What is meant by data redundancy in a database system?</p>", "Some information is stored repeatedly.", "<p>Some information is stored repeatedly.</p>", 10),
          ("What is a functional dependency (FD)?", "<p>What is a functional dependency (FD)?</p>", "A FD is a constraint between two sets of attributes in a relation from a database.", "<p>A FD is a constraint between two sets of attributes in a relation from a database.</p>", 10),
          ("What is an algorithm?", "<p>What is an algorithm?</p>", "A step by step method of solving a problem.", "<p>A step by step method of solving a problem.</p>", 4),
          ("Worst time complexity for bubble sort?", "<p>Worst time complexity for bubble sort?</p>", "O(n^2).", "<p>O(n^2).</p>", 4);


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