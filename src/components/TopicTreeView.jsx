import React from "react";
import { Link } from "react-router-dom";

import db from "./../model/database";

export default class TopicTreeView extends React.Component {
  buildTree(topics, parent_id) {
    const map = new Map();
    const tree = new Array();

    topics.forEach(t => {
      map.set(t.id, t);
      map.get(t.id).subtopics = new Array();
    });

    topics.forEach(t => {
      if (t.parent_id === parent_id) {
        tree.push(t);
      } else {
        map.get(t.parent_id).subtopics.push(t);
      }
    });

    return tree;
  }

  render(props) {
    const parent = this.props.parent;

    const topicTree = this.buildTree(
      db.getSubtopicsRecursively(parent.id),
      parent.id
    );
    topicTree.sort((a, b) => (a.sort_order > b.sort_order ? 1 : -1));
    const topicTreeList = topicTree.map(t => (
      <TopicTreeItem key={t.id} topic={t} />
    ));

    return <ul>{topicTreeList}</ul>;
  }
}

class TopicTreeItem extends React.Component {
  render(props) {
    let subtopics = null;

    if (this.props.topic.subtopics) {
      subtopics = this.props.topic.subtopics.map(t => (
        <TopicTreeItem key={t.id} topic={t} />
      ));
    }

    return (
      <li key={this.props.topic.id}>
        <span className="sort-order">{this.props.topic.sort_order}</span>
        <Link to={`/topic/${this.props.topic.id}`}>
          {this.props.topic.name}
        </Link>
        <span className="due-cards">{db.getDueCardsLength(this.props.topic.id)}</span>
        {subtopics ? <ul>{subtopics}</ul> : null}
      </li>
    );
  }
}
