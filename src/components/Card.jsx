import React from 'react';
import { Link } from 'react-router-dom';

import db from './../model/database';

export default class Card extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      frontHTML: this.props.card.front_html,
      backHTML: this.props.card.back_html,
      cardId: this.props.card.id,
      topicId: this.props.card.topic_id,
      showDropdown: false,
      renderCard: true,
    };
    this.toggleDropdown = this.toggleDropdown.bind(this);
    this.delete = this.delete.bind(this);
  }

  toggleDropdown() {
    this.setState({ showDropdown: !this.state.showDropdown });
  }

  delete() {
    db.deleteCard(this.state.cardId);
    this.setState({ renderCard: false });
  }

  render() {
    if (!this.state.renderCard) {
      return null;
    }

    const cardHTML = `${this.state.frontHTML}<hr/>${this.state.backHTML}`;

    return (
      <div className="card card-content">
        {/* eslint-disable react/no-danger */}
        <div dangerouslySetInnerHTML={{ __html: cardHTML }} />
        <Link to={`/card/submit/${this.state.cardId}/${this.state.topicId}`} className="button">
          EDIT
        </Link>

        <button
          onClick={this.toggleDropdown}
          className="button button-clear button-more"
          type="submit"
        >
          <i className="material-icons">more_vert</i>
        </button>
        <div className={this.state.showDropdown ? 'dropdown-show' : 'dropdown-hide'}>
          <hr />
          <input type="submit" value="Delete" className="button" onClick={this.delete} />
        </div>
      </div>
    );
  }
}
