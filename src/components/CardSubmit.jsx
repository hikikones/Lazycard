import React from "react";

import db from "./../model/database";

import Card from "./Card";
import TopicsSelect from "./TopicsSelect";

export default class TopicSubmit extends React.Component {
  constructor(props) {
    super(props);
    this.save = this.save.bind(this);
    this.cancel = this.cancel.bind(this);
    this.goBack = this.goBack.bind(this);

    this.front = React.createRef();
    this.back = React.createRef();

    const cardId = parseInt(this.props.match.params.card_id);
    const topicId = parseInt(this.props.match.params.topic_id);
    const currentCard = db.getCard(cardId);

    this.state = {
      card_id: cardId,
      topic_id: topicId,
      card: currentCard ? currentCard : { front: "", back: "" }
    };
  }

  componentDidMount() {
    this.handleInput();
  }

  handleInput() {
    this.resizeTextarea();
    this.renderMarkdown();
  }

  renderMarkdown() {
    const frontMD = this.front.current.value;
    const backMD = this.back.current.value;
    this.setState({ card: { front: frontMD, back: backMD } });
  }

  resizeTextarea() {
    this.front.current.style.height = "inherit";
    this.front.current.style.height = `${this.front.current.scrollHeight}px`;
    this.back.current.style.height = "inherit";
    this.back.current.style.height = `${this.back.current.scrollHeight}px`;
  }

  clearInput() {
    this.front.current.value = "";
    this.back.current.value = "";
  }

  save() {
    const frontMD = this.front.current.value;
    const backMD = this.back.current.value;
    const topicId = parseInt(document.getElementById("parent").value);

    if (this.state.card_id) {
      db.updateCard(
        this.state.card_id,
        frontMD,
        backMD,
        topicId
      );
      this.goBack();
      return;
    }

    db.createCard(frontMD, backMD, topicId);

    this.clearInput();
    this.renderMarkdown();
    this.resizeTextarea();
  }

  cancel() {
    this.goBack();
  }

  goBack() {
    this.props.history.goBack();
  }

  render() {
    return (
      <div className="card-editor-container">
        <div className="card-editor">
          <h4>Front</h4>
          <textarea
            id="front"
            ref={this.front}
            onInput={() => this.handleInput()}
            defaultValue={this.state.card.front}
          />

          <h4>Back</h4>
          <textarea
            id="back"
            ref={this.back}
            onInput={() => this.handleInput()}
            defaultValue={this.state.card.back}
          />

          <TopicsSelect
            card_id={this.state.card_id}
            topic_id={this.state.topic_id}
          />

          <input
            onClick={() => this.save()}
            type="submit"
            className="button-primary float-left"
            value="Save"
          />

          <input
            onClick={() => this.cancel()}
            type="submit"
            className="button-primary float-right"
            value="Cancel"
          />
        </div>
        <div>
          <h4>Card preview</h4>
          <Card card={this.state.card} renderButtons={false} />
        </div>
      </div>
    );
  }
}
