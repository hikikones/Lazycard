import fs from "fs";
import path from "path";
import sqlite3 from "better-sqlite3";

import Config from "./../controller/Config";

import schema from "./schema.sql";

class Database {
  constructor() {
    const dbPath = path.resolve(Config.getUserDataPath(), "database.db");
    const dbFileExists = fs.existsSync(dbPath);
    this.db = new sqlite3(dbPath);
    if (!dbFileExists) {
      this.db.exec(schema);
    }
  }

  // TOPICS
  getTopicId(topicName) {
    const topic = this.db
      .prepare("SELECT id FROM topics WHERE name = ?")
      .get(topicName);
    return topic.id;
  }

  getTopic(topicId) {
    const topic = this.db
      .prepare("SELECT * FROM topics WHERE id = ?")
      .get(topicId);
    return topic;
  }

  getRootTopics() {
    const rootTopics = this.db
      .prepare("SELECT * FROM topics WHERE parent_id IS NULL")
      .all();
    return rootTopics;
  }

  getTopics() {
    const topics = this.db.prepare("SELECT * FROM topics").all();
    return topics;
  }

  getTopicsExcept(topicId) {
    const topics = this.db
      .prepare("SELECT * FROM topics WHERE id != ?")
      .all(topicId);
    return topics;
  }

  getSubtopics(topicId) {
    const subtopics = this.db
      .prepare("SELECT * FROM topics WHERE parent_id = ?")
      .all(topicId);
    return subtopics;
  }

  getSubtopicsRecursively(topicId) {
    const sql = `WITH RECURSIVE child_topics (id, name, parent_id) AS (
                SELECT id, name, parent_id
                FROM topics
                WHERE parent_id = ?
                UNION ALL
                SELECT t.id, t.name, t.parent_id
                FROM topics t
                INNER JOIN child_topics ON t.parent_id = child_topics.id
                )
                SELECT * FROM child_topics;`;
    return this.db.prepare(sql).all(topicId);
  }

  createTopic(name, image, parentId) {
    const create = `INSERT INTO topics (name, image, parent_id)
                    VALUES (?, ?, ?)`;
    this.db.prepare(create).run(name, image, parentId);
  }

  updateTopic(topicId, newName, newImage, newParentId) {
    const updateTopic = `UPDATE topics
                          SET name = ?, image = ?, parent_id = ?
                        WHERE id = ?`;
    if (this.isSubtopic(newParentId, topicId)) {
      const topicParentId = this.db
        .prepare("SELECT parent_id FROM topics WHERE id = ?")
        .get(topicId).parent_id;
      this.db
        .prepare("UPDATE topics SET parent_id = ? WHERE id = ?")
        .run(topicParentId, newParentId);
    }
    this.db.prepare(updateTopic).run(newName, newImage, newParentId, topicId);
  }

  deleteTopic(topicId) {
    this.db.prepare("DELETE FROM topics WHERE id = ?").run(topicId);
  }

  isSubtopic(topicId, possibleParentTopicId) {
    if (!topicId || !possibleParentTopicId) {
      return false;
    }

    const parent = this.getTopic(possibleParentTopicId);
    let topic = this.getTopic(topicId);

    while (topic.parent_id) {
      if (topic.parent_id === parent.id) {
        return true;
      }
      topic = this.getTopic(topic.parent_id);
    }

    return false;
  }

  // CARDS
  getCard(cardId) {
    return this.db.prepare("SELECT * FROM cards WHERE id = ?").get(cardId);
  }

  getCards(topicId) {
    const cards = this.db
      .prepare("SELECT * FROM cards WHERE topic_id = ?")
      .all(topicId);
    return cards;
  }

  getDueCards(topicId) {
    const subtopics = this.getSubtopicsRecursively(topicId);
    subtopics.push({ id: topicId });
    const cards = `SELECT * FROM cards WHERE topic_id IN
                  (${subtopics.map(t => t.id).join(",")})
                  AND due_date <= (date('now'))
                  ORDER BY due_date ASC`;
    return this.db.prepare(cards).all();
  }

  createCard(frontMD, frontHTML, backMD, backHTML, topicId) {
    const create = `INSERT INTO cards (front_md, front_html, back_md, back_html, topic_id)
                    VALUES (?, ?, ?, ?, ?)`;
    this.db.prepare(create).run(frontMD, frontHTML, backMD, backHTML, topicId);
  }

  updateCard(cardId, frontMD, frontHTML, backMD, backHTML, topicId) {
    const update = `UPDATE cards
                      SET front_md = ?, front_html = ?, back_md = ?, back_html = ?, topic_id = ?
                    WHERE id = ?`;
    this.db
      .prepare(update)
      .run(frontMD, frontHTML, backMD, backHTML, topicId, cardId);
  }

  updateCardReview(cardId, dueDate, dueDays) {
    const update = `UPDATE cards
                      SET due_date = ?, due_days = ?, last_review_date = (date('now'))
                    WHERE id = ?`;
    this.db.prepare(update).run(dueDate, dueDays, cardId);
  }

  deleteCard(cardId) {
    this.db.prepare("DELETE FROM cards WHERE id = ?").run(cardId);
  }
}

export default new Database();
