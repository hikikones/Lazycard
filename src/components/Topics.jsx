import React from 'react';
import db from './../model/database';

import TopicCard from './TopicCard';
import Card from './Card';

export default class Topics extends React.Component {
  renderTopics(topicId) {
    const topics = topicId === 0 ? db.getRootTopics() : db.getSubtopics(topicId);
    const title = topicId === 0 ? 'Topics' : db.getTopic(topicId).name;
    const topicList = topics.map(t => <TopicCard key={t.id} topic={t} />);
    return (
      <div>
        <h1 className="text-center">{title}</h1>
        <div className="cards-container">{topicList}</div>
      </div>
    );
  }

  renderCards(topicId) {
    const cards = db.getCards(topicId);
    if (cards.length === 0) {
      return null;
    }

    const cardsList = cards.map(c => <Card key={c.id} card={c} />);
    return (
      <div>
        <h2 className="text-center"> Cards</h2>
        <div className="cards-container">{cardsList}</div>
      </div>
    );
  }

  render(props) {
    const topicId = parseInt(this.props.match.params.topic_id);
    return (
      <div>
        {this.renderTopics(topicId)}
        {this.renderCards(topicId)}
      </div>
    );
  }
}
