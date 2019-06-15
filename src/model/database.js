import fs from "fs";
import path from "path";
import sqlite3 from "better-sqlite3";
import archiver from "archiver";

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
    // TODO - Main and renderer process both create an instance. This should not happen.
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
    const sql = `WITH RECURSIVE child_topics (id, name, sort_order, parent_id) AS (
                SELECT id, name, sort_order, parent_id
                FROM topics
                WHERE parent_id = ?
                UNION ALL
                SELECT t.id, t.name, t.sort_order, t.parent_id
                FROM topics t
                INNER JOIN child_topics ON t.parent_id = child_topics.id
                )
                SELECT * FROM child_topics;`;
    return this.db.prepare(sql).all(topicId);
  }

  createTopic(name, image, sortOrder, parentId) {
    const create = `INSERT INTO topics (name, image, sort_order, parent_id)
                    VALUES (?, ?, ?, ?)`;
    this.db.prepare(create).run(name, image, sortOrder, parentId);
    this.backupQuick();
  }

  updateTopic(topicId, newName, newImage, newSortOrder, newParentId) {
    const updateTopic = `UPDATE topics
                          SET name = ?, image = ?, sort_order = ?, parent_id = ?
                        WHERE id = ?`;
    if (this.isSubtopic(newParentId, topicId)) {
      const topicParentId = this.db
        .prepare("SELECT parent_id FROM topics WHERE id = ?")
        .get(topicId).parent_id;
      this.db
        .prepare("UPDATE topics SET parent_id = ? WHERE id = ?")
        .run(topicParentId, newParentId);
    }
    this.db
      .prepare(updateTopic)
      .run(newName, newImage, newSortOrder, newParentId, topicId);
    this.backupQuick();
  }

  deleteTopic(topicId) {
    this.db.prepare("DELETE FROM topics WHERE id = ?").run(topicId);
    this.backupQuick();
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

  getAllCards() {
    return this.db.prepare("SELECT * FROM cards").all();
  }

  getAllCardsLength() {
    return this.db.prepare("SELECT COUNT(id) AS length FROM cards").get().length;
  }

  getCardsRecursively(topicId) {
    const subtopics = this.getSubtopicsRecursively(topicId);
    subtopics.push({ id: topicId });
    const cards = `SELECT * FROM cards WHERE topic_id IN
                  (${subtopics.map(t => t.id).join(",")})`;
    return this.db.prepare(cards).all();
  }

  getCardsRecursivelyLength(topicId) {
    const subtopics = this.getSubtopicsRecursively(topicId);
    subtopics.push({ id: topicId });
    const cards = `SELECT COUNT(id) AS length FROM cards WHERE topic_id IN
                  (${subtopics.map(t => t.id).join(",")})`;
    return this.db.prepare(cards).get().length;
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

  getDueCardsLength(topicId) {
    const subtopics = this.getSubtopicsRecursively(topicId);
    subtopics.push({ id: topicId });
    const cards = `SELECT COUNT(id) AS length FROM cards WHERE topic_id IN
                  (${subtopics.map(t => t.id).join(",")})
                  AND due_date <= (date('now'))`;
    return this.db.prepare(cards).get().length;
  }

  getAllDueCards() {
    const dueCards = `SELECT * FROM cards WHERE due_date <= (date('now'))`;
    return this.db.prepare(dueCards).all();
  }

  getAllDueCardsLength() {
    const dueCardsLength = `SELECT due_date, COUNT(id) AS length FROM cards
                            WHERE due_date <= (date('now'))`;
    return this.db.prepare(dueCardsLength).get().length;
  }

  getCardsByKeywords(text) {
    const words = text.split(" ");
    let search = `SELECT *, front || " " || back AS text FROM cards
                  WHERE text LIKE ?`;
    if (words.length > 1) {
      for (let i = 0; i < words.length - 1; i++) {
        search += ` AND text LIKE ?`;
      }
    }
    return this.db.prepare(search).all(words.map(w => `%${w}%`));
  }

  createCard(front, back, topicId) {
    const create = `INSERT INTO cards (front, back, topic_id)
                    VALUES (?, ?, ?)`;
    this.db.prepare(create).run(front, back, topicId);
    this.backupQuick();
  }

  updateCard(cardId, front, back, topicId) {
    const update = `UPDATE cards
                      SET front = ?, back = ?, topic_id = ?
                    WHERE id = ?`;
    this.db
      .prepare(update)
      .run(front, back, topicId, cardId);
    this.backupQuick();
  }

  updateCardReview(cardId, dueDate, dueDays) {
    const update = `UPDATE cards
                      SET due_date = ?, due_days = ?, last_review_date = (date('now'))
                    WHERE id = ?`;
    this.db.prepare(update).run(dueDate, dueDays, cardId);
    this.backupQuick();
  }

  deleteCard(cardId) {
    this.db.prepare("DELETE FROM cards WHERE id = ?").run(cardId);
    this.backupQuick();
  }

  // BACKUP
  backupQuick() {
    this.db.backup(path.join(Config.getBackupsPath(), `backup-${Date.now()}.db`));
  }

  backup(callback) {
    var output = fs.createWriteStream(Config.get("backup"));
    var archive = archiver("zip", { forceLocalTime: true });

    output.on("close", function() {
      callback();
    });

    archive.pipe(output);
    archive.file(this.db.name, { name: "database.db" });
    archive.directory(Config.getImagesPath(), "images");
    archive.finalize();
  }
}

export default new Database();
