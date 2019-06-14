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

  sortTree(topics) {
    if (topics.length === 0) {
      return [];
    }

    topics.forEach(t => {
      t.subtopics = this.sortTree(t.subtopics);
    });

    const topicsSorted = topics.sort((a, b) => {
      if (a.sort_order && b.sort_order) {
        return a.sort_order > b.sort_order ? 1 : -1;
      } else {
        return a.name > b.name ? 1 : -1;
      }
    });

    return topicsSorted;
  }

  render(props) {
    const parent = this.props.parent;

    let topicTree = this.buildTree(
      db.getSubtopicsRecursively(parent.id),
      parent.id
    );

    topicTree = this.sortTree(topicTree);

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
        {this.props.topic.sort_order ? (
          <span className="sort-order">{this.props.topic.sort_order}</span>
        ) : null}
        <Link to={`/topic/${this.props.topic.id}`}>
          {this.props.topic.name}
        </Link>
        <span className="due-cards">
          {db.getDueCardsLength(this.props.topic.id)}
        </span>
        {subtopics ? <ul>{subtopics}</ul> : null}
      </li>
    );
  }
}
