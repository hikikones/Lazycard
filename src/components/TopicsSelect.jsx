import React from "react";
import db from "../model/database";

export default class TopicsDatalist extends React.Component {
  constructor(props) {
    super(props);
    this.getDefaultValue = this.getDefaultValue.bind(this);
  }

  getDefaultValue(topicId) {
    if (topicId === 0) {
      return 0;
    }
    if (this.props.card_id) {
      return topicId;
    }
    const topic = db.getTopic(topicId);
    return topic.parent_id ? topic.parent_id : 0;
  }

  render(props) {
    const topicId = this.props.topic_id;
    const topics =
      topicId === 0 || this.props.card_id
        ? db.getTopics()
        : db.getTopicsExcept(topicId);

    const cardId = this.props.card_id;

    if (cardId === null || cardId === undefined) {
      topics.unshift({ id: 0, name: "--" });
    }

    const topicList = topics.map(t => (
      <option key={t.id} value={t.id}>
        {t.name}
      </option>
    ));

    return (
      <div>
        <label htmlFor="parent">Parent</label>
        <select id="parent" defaultValue={this.getDefaultValue(topicId)}>
          {topicList}
        </select>
      </div>
    );
  }
}
