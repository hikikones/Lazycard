CREATE TABLE topics (
  id INTEGER,
  name TEXT NOT NULL,
  image TEXT,
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

INSERT INTO topics (name, image, parent_id)
  VALUES  ("Computer Science", "https://images-na.ssl-images-amazon.com/images/I/51rPLfOvqxL._SX376_BO1,204,203,200_.jpg", NULL),
          ("Mathematics", NULL, NULL),
          ("Physics", NULL, NULL),
          ("Algorithms", NULL, 1),
          ("Sorting", NULL, 4),
          ("Data Structures", NULL, 4),
          ("Graphs", NULL, 4),
          ("Hash Tables", NULL, 6),
          ("Databases", NULL, 1),
          ("Normalization", NULL, 9),
          ("SQL", NULL, 9),
          ("Networking", NULL, 1),
          ("Big-O", NULL, 4),
          ("Advanced", NULL, 8);

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