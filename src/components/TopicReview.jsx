import React from 'react';
import { Link } from 'react-router-dom';

import db from './../model/database';

export default class TopicReview extends React.Component {
  constructor(props) {
    super(props);

    this.renderCard = this.renderCard.bind(this);
    this.renderFront = this.renderFront.bind(this);
    this.renderBack = this.renderBack.bind(this);
    this.renderButtons = this.renderButtons.bind(this);
    this.getNextCard = this.getNextCard.bind(this);
    this.toggleAnswer = this.toggleAnswer.bind(this);
    this.handleAnswer = this.handleAnswer.bind(this);
    this.getDueDays = this.getDueDays.bind(this);
    this.daysBetween = this.daysBetween.bind(this);
    this.toggleDropdown = this.toggleDropdown.bind(this);
    this.showNextCard = this.showNextCard.bind(this);
    this.delete = this.delete.bind(this);

    this.state = {
      topicId: parseInt(this.props.match.params.topic_id),
      showAnswer: false,
      showDropdown: false,
    };

    this.cards = db.getDueCards(this.state.topicId);
    this.card = this.getNextCard();
  }

  getNextCard() {
    return this.cards.pop();
  }

  delete() {
    db.deleteCard(this.card.id);
    this.toggleDropdown();
    this.showNextCard();
  }

  toggleAnswer() {
    this.setState({ showAnswer: !this.state.showAnswer });
  }

  toggleDropdown() {
    this.setState({ showDropdown: !this.state.showDropdown });
  }

  handleAnswer(answeredCorrectly) {
    const newDueDays = this.getDueDays(answeredCorrectly);
    const newDueDate = this.addDays(
      this.card.last_review_date
        ? new Date(this.card.last_review_date)
        : new Date(this.card.due_date),
      newDueDays,
    );
    db.updateCardReview(this.card.id, newDueDate, newDueDays);

    if (!answeredCorrectly) {
      this.cards.unshift(this.card);
    }

    this.showNextCard();
  }

  showNextCard() {
    this.card = this.getNextCard();
    this.toggleAnswer();
  }

  getDueDays(answeredCorrectly) {
    if (!answeredCorrectly) {
      return 0;
    }

    const doubleCount = 1 + this.getFactorCount(this.getOverdueFactor(this.card));
    let dueDays = this.card.due_days;
    for (let i = 0; i < doubleCount; i++) {
      if (dueDays === 0) {
        dueDays = 1;
      } else {
        dueDays *= 2;
      }
    }

    return dueDays;
  }

  getOverdueFactor() {
    if (!this.card.last_review_date) {
      return this.daysBetween(new Date(Date.now()), new Date(this.card.due_date));
    }
    if (this.card.due_days === 0) {
      return this.daysBetween(new Date(Date.now()), new Date(this.card.last_review_date));
    }
    const daysOverdue = this.daysBetween(
      new Date(Date.now()),
      new Date(this.card.last_review_date),
    );
    return Math.round(daysOverdue / this.card.due_days);
  }

  getFactorCount(num) {
    let power = 0;
    let factor = 1;
    while (factor * 2 <= num) {
      factor *= 2;
      power++;
    }
    return power;
  }

  addDays(date, days) {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return this.formatDate(result);
  }

  daysBetween(date1, date2) {
    // The number of milliseconds in one day
    const ONE_DAY = 1000 * 60 * 60 * 24;

    // Convert both dates to milliseconds
    const date1_ms = date1.getTime();
    const date2_ms = date2.getTime();

    // Calculate the difference in milliseconds
    const difference_ms = Math.abs(date1_ms - date2_ms);

    // Convert back to days and return
    return Math.round(difference_ms / ONE_DAY);
  }

  formatDate(date) {
    const d = new Date(date);
    let month = `${d.getMonth() + 1}`;
    let day = `${d.getDate()}`;
    const year = d.getFullYear();

    if (month.length < 2) month = `0${month}`;
    if (day.length < 2) day = `0${day}`;

    return [year, month, day].join('-');
  }

  renderCard() {
    return (
      <div>
        {this.renderFront()}
        {this.renderBack()}
        {this.renderButtons()}
      </div>
    );
  }

  renderFront() {
    return <div dangerouslySetInnerHTML={{ __html: this.card.front_html }} />;
  }

  renderBack() {
    if (!this.state.showAnswer) {
      return null;
    }
    return (
      <div>
        <hr />
        <div dangerouslySetInnerHTML={{ __html: this.card.back_html }} />

        <Link to={`/card/submit/${this.card.id}/${this.state.topicId}`} className="button">
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

  renderButtons() {
    if (!this.state.showAnswer) {
      return (
        <div className="review-buttons">
          <input type="submit" value="SHOW ANSWER" className="button" onClick={this.toggleAnswer} />
        </div>
      );
    }
    return (
      <div>
        <hr />
        <div className="review-buttons">
          <button
            onClick={() => this.handleAnswer(false)}
            className="button-clear button-review-answer"
            type="submit"
          >
            <i className="material-icons">thumb_down</i>
          </button>
          <button
            onClick={() => this.handleAnswer(true)}
            className="button-clear button-review-answer"
            type="submit"
          >
            <i className="material-icons">thumb_up</i>
          </button>
        </div>
      </div>
    );
  }

  render() {
    if (!this.card) {
      return <h2>Good job!</h2>;
    }

    return <div className="card card-content">{this.renderCard()}</div>;
  }
}

/*
import React from 'react';
import db from './../model/database';

export default class TopicReview extends React.Component {
  constructor(props) {
    super(props);

    this.renderCard = this.renderCard.bind(this);
    this.renderFront = this.renderFront.bind(this);
    this.renderBack = this.renderBack.bind(this);
    this.renderButtons = this.renderButtons.bind(this);
    this.getNextCard = this.getNextCard.bind(this);
    this.toggleAnswer = this.toggleAnswer.bind(this);
    this.handleAnswer = this.handleAnswer.bind(this);
    this.getDueDays = this.getDueDays.bind(this);
    this.daysBetween = this.daysBetween.bind(this);

    this.state = {
      topicId: parseInt(this.props.match.params.topic_id),
      showAnswer: false,
    };

    this.cards = db.getDueCards(this.state.topicId);
    this.card = this.getNextCard();
  }

  renderCard() {
    return (
      <div>
        {this.renderFront()}
        {this.renderBack()}
        {this.renderButtons()}
      </div>
    );
  }

  renderFront() {
    return <div dangerouslySetInnerHTML={{ __html: this.card.front_html }} />;
  }

  renderBack() {
    if (!this.state.showAnswer) {
      return null;
    }
    return (
      <div>
        <hr />
        <div dangerouslySetInnerHTML={{ __html: this.card.back_html }} />
      </div>
    );
  }

  renderButtons() {
    if (!this.state.showAnswer) {
      return (
        <div className="review-buttons">
          <input type="submit" value="SHOW ANSWER" className="button" onClick={this.toggleAnswer} />
        </div>
      );
    }
    return (
      <div className="review-buttons">
        <button
          onClick={() => this.handleAnswer(false)}
          className="button-clear button-review-answer"
          type="submit"
        >
          <i className="material-icons">thumb_down</i>
        </button>
        <button
          onClick={() => this.handleAnswer(true)}
          className="button-clear button-review-answer"
          type="submit"
        >
          <i className="material-icons">thumb_up</i>
        </button>
      </div>
    );
  }

  getNextCard() {
    return this.cards.pop();
  }

  toggleAnswer() {
    this.setState({ showAnswer: !this.state.showAnswer });
  }

  handleAnswer(answeredCorrectly) {
    const newDueDays = this.getDueDays(answeredCorrectly);
    const newDueDate = this.addDays(
      this.card.last_review_date
        ? new Date(this.card.last_review_date)
        : new Date(this.card.due_date),
      newDueDays,
    );
    db.updateCardReview(this.card.id, newDueDate, newDueDays);

    if (!answeredCorrectly) {
      this.cards.unshift(this.card);
    }

    this.card = this.getNextCard();
    this.toggleAnswer();
  }

  getDueDays(answeredCorrectly) {
    if (!answeredCorrectly) {
      return 0;
    }

    const doubleCount = 1 + this.getFactorCount(this.getOverdueFactor(this.card));
    let dueDays = this.card.due_days;
    for (let i = 0; i < doubleCount; i++) {
      if (dueDays === 0) {
        dueDays = 1;
      } else {
        dueDays *= 2;
      }
    }

    return dueDays;
  }

  getOverdueFactor() {
    if (!this.card.last_review_date) {
      return this.daysBetween(new Date(Date.now()), new Date(this.card.due_date));
    }
    if (this.card.due_days === 0) {
      return this.daysBetween(new Date(Date.now()), new Date(this.card.last_review_date));
    }
    const daysOverdue = this.daysBetween(
      new Date(Date.now()),
      new Date(this.card.last_review_date),
    );
    return Math.round(daysOverdue / this.card.due_days);
  }

  getFactorCount(num) {
    let power = 0;
    let factor = 1;
    while (factor * 2 <= num) {
      factor *= 2;
      power++;
    }
    return power;
  }

  addDays(date, days) {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return this.formatDate(result);
  }

  daysBetween(date1, date2) {
    // The number of milliseconds in one day
    const ONE_DAY = 1000 * 60 * 60 * 24;

    // Convert both dates to milliseconds
    const date1_ms = date1.getTime();
    const date2_ms = date2.getTime();

    // Calculate the difference in milliseconds
    const difference_ms = Math.abs(date1_ms - date2_ms);

    // Convert back to days and return
    return Math.round(difference_ms / ONE_DAY);
  }

  formatDate(date) {
    const d = new Date(date);
    let month = `${d.getMonth() + 1}`;
    let day = `${d.getDate()}`;
    const year = d.getFullYear();

    if (month.length < 2) month = `0${month}`;
    if (day.length < 2) day = `0${day}`;

    return [year, month, day].join('-');
  }

  render() {
    if (!this.card) {
      return <h2>Good job!</h2>;
    }

    return <div className="card card-content">{this.renderCard()}</div>;
  }
}
*/

/*
import React from 'react';
import db from './../model/database';

export default class TopicReview extends React.Component {
  constructor(props) {
    super(props);

    this.renderCard = this.renderCard.bind(this);
    this.renderFront = this.renderFront.bind(this);
    this.renderBack = this.renderBack.bind(this);
    this.renderButtons = this.renderButtons.bind(this);
    this.getNextCard = this.getNextCard.bind(this);
    this.toggleAnswer = this.toggleAnswer.bind(this);
    this.handleAnswer = this.handleAnswer.bind(this);
    this.getDueDays = this.getDueDays.bind(this);
    this.getDateNow = this.getDateNow.bind(this);
    this.daysBetween = this.daysBetween.bind(this);

    this.state = {
      topicId: parseInt(this.props.match.params.topic_id),
      showAnswer: false,
      today: this.getDateNow(),
    };

    this.cards = db.getCards(this.state.topicId);
    this.card = this.getNextCard();
  }

  renderCard() {
    return (
      <div>
        {this.renderFront()}
        {this.renderBack()}
        {this.renderButtons()}
      </div>
    );
  }

  renderFront() {
    return <div dangerouslySetInnerHTML={{ __html: this.card.front_html }} />;
  }

  renderBack() {
    if (!this.state.showAnswer) {
      return null;
    }
    return (
      <div>
        <hr />
        <div dangerouslySetInnerHTML={{ __html: this.card.back_html }} />
      </div>
    );
  }

  renderButtons() {
    if (!this.state.showAnswer) {
      return (
        <div className="review-buttons">
          <input type="submit" value="SHOW ANSWER" className="button" onClick={this.toggleAnswer} />
        </div>
      );
    }
    return (
      <div className="review-buttons">
        <button
          onClick={() => this.handleAnswer(false)}
          className="button-clear button-review-answer"
          type="submit"
        >
          <i className="material-icons">thumb_down</i>
        </button>
        <button
          onClick={() => this.handleAnswer(true)}
          className="button-clear button-review-answer"
          type="submit"
        >
          <i className="material-icons">thumb_up</i>
        </button>
      </div>
    );
  }

  getNextCard() {
    return this.cards.pop();
  }

  toggleAnswer() {
    this.setState({ showAnswer: !this.state.showAnswer });
  }

  handleAnswer(answeredCorrectly) {
    const card = db.getCard(this.card.id);

    // TODO
    console.log(`DUE DATE: ${card.due_days}`);
    const newDueDays = this.getDueDays(card, answeredCorrectly);
    console.log(`NEW DUE DAYS: ${newDueDays}`);
    const lastDate = card.last_review_date ? card.last_review_date : 'NULL';
    console.log(`LAST DATE: ${lastDate}`);
    console.log(`DUE DATE: ${card.due_date}`);
    const newDueDate = this.addDays(
      card.last_review_date ? new Date(card.last_review_date) : new Date(card.due_date),
      newDueDays,
    );
    console.log(`NEW DUE DATE: ${newDueDate}`);

    db.updateCardReview(card.id, newDueDate, newDueDays);

    this.card = this.getNextCard();
    this.toggleAnswer();
  }

  getDueDays(card, answeredCorrectly) {
    if (!answeredCorrectly) {
      return 0;
    }

    const doubleCount = 1 + this.getFactorCount(this.getOverdueFactor(card));
    let dueDays = card.due_days === 0 ? 1 : card.due_days;
    for (let i = 0; i < doubleCount; i++) {
      dueDays *= 2;
    }

    return dueDays;
  }

  getOverdueFactor(card) {
    if (!card.last_review_date) {
      return this.daysBetween(new Date(this.state.today), new Date(card.due_date));
    }
    if (card.due_days === 0) {
      return this.daysBetween(new Date(this.state.today), new Date(card.last_review_date));
    }
    const daysOverdue = this.daysBetween(
      new Date(this.state.today),
      new Date(card.last_review_date),
    );
    return Math.round(daysOverdue / card.due_days);
  }

  getFactorCount(num) {
    let power = 0;
    let factor = 1;
    while (factor * 2 <= num) {
      factor *= 2;
      power++;
    }
    return power;
  }

  addDays(date, days) {
    const result = new Date(date);
    result.setDate(result.getDate() + days);
    return this.formatDate(result);
  }

  daysBetween(date1, date2) {
    // The number of milliseconds in one day
    const ONE_DAY = 1000 * 60 * 60 * 24;

    // Convert both dates to milliseconds
    const date1_ms = date1.getTime();
    const date2_ms = date2.getTime();

    // Calculate the difference in milliseconds
    const difference_ms = Math.abs(date1_ms - date2_ms);

    // Convert back to days and return
    return Math.round(difference_ms / ONE_DAY);
  }

  getDateNow() {
    const d = new Date();
    let month = `${d.getMonth() + 1}`;
    let day = `${d.getDate()}`;
    const year = d.getFullYear();

    if (month.length < 2) month = `0${month}`;
    if (day.length < 2) day = `0${day}`;

    return [year, month, day].join('-');
  }

  formatDate(date) {
    const d = new Date(date);
    let month = `${d.getMonth() + 1}`;
    let day = `${d.getDate()}`;
    const year = d.getFullYear();

    if (month.length < 2) month = `0${month}`;
    if (day.length < 2) day = `0${day}`;

    return [year, month, day].join('-');
  }

  render() {
    if (!this.card) {
      return (
        <div>
          <h2>Good job!</h2>
        </div>
      );
    }

    return <div className="card card-content">{this.renderCard()}</div>;
  }
}
*/
