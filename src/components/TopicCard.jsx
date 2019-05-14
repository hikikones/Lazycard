import path from "path";
import React from "react";
import { Link } from "react-router-dom";

import db from "./../model/database";
import Config from "./../controller/Config";

import TopicTreeView from "./TopicTreeView";

export default class Topic extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      showDropdown: false,
      renderTopic: true
    };
    this.toggleDropdown = this.toggleDropdown.bind(this);
    this.delete = this.delete.bind(this);
  }

  renderImage(topic) {
    if (!topic.image) {
      return null;
    }
    const image = path.join(Config.getImagesPath(), topic.image);
    return <img src={image} alt="" />;
  }

  renderTopics(topic) {
    return (
      <div className="card-content">
        <h3>
          <Link to={`/topic/${topic.id}`}>{topic.name}</Link>
        </h3>

        <TopicTreeView parent={topic} />

        <Link to={`/topic/review/${topic.id}`} className="button">
          REVIEW
        </Link>

        <Link to={`/topic/submit/${topic.id}`} className="button">
          EDIT
        </Link>

        <button
          onClick={this.toggleDropdown}
          className="button button-clear button-more"
          type="submit"
        >
          <i className="material-icons">more_vert</i>
        </button>
        <div
          className={
            this.state.showDropdown ? "dropdown-show" : "dropdown-hide"
          }
        >
          <hr />
          <input
            type="submit"
            value="Delete"
            className="button"
            onClick={() => this.delete(topic.id)}
          />
        </div>
      </div>
    );
  }

  toggleDropdown() {
    this.setState({ showDropdown: !this.state.showDropdown });
  }

  delete(topicId) {
    db.deleteTopic(topicId);
    this.setState({ renderTopic: false });
  }

  render(props) {
    if (!this.state.renderTopic) {
      return null;
    }

    const topic = this.props.topic;

    return (
      <div className="topic card">
        {this.renderImage(topic)}
        {this.renderTopics(topic)}
      </div>
    );
  }
}
