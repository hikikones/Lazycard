import React from "react";
import { Link } from "react-router-dom";

import db from "./../model/database";

import Card from "./Card";

export default class TopicReview extends React.Component {
  constructor(props) {
    super(props);

    this.renderAnswerButtons = this.renderAnswerButtons.bind(this);
    this.getNextCard = this.getNextCard.bind(this);
    this.toggleAnswer = this.toggleAnswer.bind(this);
    this.handleAnswer = this.handleAnswer.bind(this);
    this.getDueDays = this.getDueDays.bind(this);
    this.daysBetween = this.daysBetween.bind(this);
    this.toggleDropdown = this.toggleDropdown.bind(this);
    this.showNextCard = this.showNextCard.bind(this);
    this.delete = this.delete.bind(this);
    this.activateCustomStudy = this.activateCustomStudy.bind(this);

    const topic_id = parseInt(this.props.match.params.topic_id);
    this.cards = topic_id === 0 ? db.getAllDueCards() : db.getDueCards(topic_id);

    if (this.cards.length >= 2) {
      this.cards = this.shuffle(this.cards);
    }

    this.state = {
      topicId: topic_id,
      showAnswer: false,
      showDropdown: false,
      custom: false,
      topicHasNoCards: topic_id === 0
        ? db.getAllCardsLength() === 0
        : db.getCardsRecursivelyLength(topic_id) === 0,
      card: this.getNextCard()
    };

    document.onkeydown = function (e) {
      e = e || window.event;
      switch(e.key) {
        case " ":
          const showAnswerBtn = document.getElementById("show-answer");
          if (showAnswerBtn) showAnswerBtn.click();
          return;
        case "ArrowDown":
          const thumbsDownBtn = document.getElementById("thumbs-down");
          if (thumbsDownBtn) thumbsDownBtn.click();
          return;
        case "ArrowUp":
          const thumbsUpBtn = document.getElementById("thumbs-up");
          if (thumbsUpBtn) thumbsUpBtn.click();
          return;
      }
    };
  }

  getNextCard() {
    return this.cards.pop();
  }

  delete() {
    db.deleteCard(this.state.card.id);
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
    if (this.state.custom) {
      this.showNextCard();
      return;
    }

    const newDueDays = this.getDueDays(answeredCorrectly);
    const newDueDate = this.addDays(
      this.state.card.last_review_date
        ? new Date(this.state.card.last_review_date)
        : new Date(this.state.card.due_date),
      newDueDays
    );
    db.updateCardReview(this.state.card.id, newDueDate, newDueDays);

    if (!answeredCorrectly) {
      this.cards.unshift(this.state.card);
    }

    this.showNextCard();
  }

  showNextCard() {
    this.setState({ card: this.getNextCard() });
    this.toggleAnswer();
  }

  shuffle(array) {
    for (let currentIndex = array.length - 1; currentIndex > 0; currentIndex--) {
        const newIndex = Math.floor(Math.random() * (currentIndex + 1));
        const temp = array[currentIndex];
        array[currentIndex] = array[newIndex];
        array[newIndex] = temp;
    }
    return array;
  }

  activateCustomStudy() {
    const topicId = this.state.topicId;
    this.cards = topicId === 0 ? db.getAllCards() : db.getCardsRecursively(topicId);
    if (this.cards.length >= 2) {
      this.cards = this.shuffle(this.cards);
    }
    this.setState({ custom: true, card: this.getNextCard() });
  }

  getDueDays(answeredCorrectly) {
    if (!answeredCorrectly) {
      return 0;
    }

    const doubleCount =
      1 + this.getFactorCount(this.getOverdueFactor(this.state.card));
    let dueDays = this.state.card.due_days;
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
    if (!this.state.card.last_review_date) {
      return this.daysBetween(
        new Date(Date.now()),
        new Date(this.state.card.due_date)
      );
    }
    if (this.state.card.due_days === 0) {
      return this.daysBetween(
        new Date(Date.now()),
        new Date(this.state.card.last_review_date)
      );
    }
    const daysOverdue = this.daysBetween(
      new Date(Date.now()),
      new Date(this.state.card.last_review_date)
    );
    return Math.round(daysOverdue / this.state.card.due_days);
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

    return [year, month, day].join("-");
  }

  renderAnswerButtons() {
    if (!this.state.showAnswer) {
      return (
        <div className="review-buttons">
          <input
            id="show-answer"
            type="submit"
            value="SHOW ANSWER"
            className="button"
            onClick={this.toggleAnswer}
          />
        </div>
      );
    }
    return (
      <div>
        <hr />
        <div className="review-buttons">
          <button
            id="thumbs-down"
            onClick={() => this.handleAnswer(false)}
            className="button-clear button-review-answer"
            type="submit"
          >
            <i className="material-icons">thumb_down</i>
          </button>
          <button
            id="thumbs-up"
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
    if (this.state.topicHasNoCards) {
      return (
        <div>
          <h2>No cards found!</h2>
          <p>This topic has no cards yet.</p>
          <Link to={`/card/submit/0/${this.state.topicId}`} className="button">
            ADD CARD
          </Link>
        </div>
      );
    }

    if (!this.state.card) {
      return (
        <div>
          <h2>Good job!</h2>
          <p>There are no more remaining cards to be reviewed.</p>
          <p>Do a custom study for reviewing as many cards as you like without affecting the scheduler.</p>
          <input
            type="submit"
            value="CUSTOM STUDY"
            className="button"
            onClick={() => this.activateCustomStudy()}
          />
        </div>
      );
    }

    return (
      <Card 
        card={this.state.card}
        renderBack={this.state.showAnswer}
        renderButtons={this.state.showAnswer}
        deleteAction={this.delete}
      >
        {this.renderAnswerButtons()}
      </Card>
    );
  }
}