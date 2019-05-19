import React from "react";
import path from "path";
import ReactHtmlParser, { convertNodeToElement } from "react-html-parser";

import db from "./../model/database";
import Config from "./../controller/Config";

import TopicsSelect from "./TopicsSelect";

const md = require("markdown-it")();
const prism = require("markdown-it-prism");
const mk = require("@iktakahiro/markdown-it-katex");

md.use(prism);
md.use(mk);

const options = {
  decodeEntities: true,
  transform
};

function transform(node, index) {
  if (node.type === "tag" && node.name === "img") {
    node.attribs.src = path.join(Config.getImagesPath(), node.attribs.src);
    return convertNodeToElement(node, index, transform);
  }
}

export default class TopicSubmit extends React.Component {
  constructor(props) {
    super(props);
    this.save = this.save.bind(this);
    this.cancel = this.cancel.bind(this);
    this.goBack = this.goBack.bind(this);

    this.front = React.createRef();
    this.back = React.createRef();
    this.preview = React.createRef();

    this.state = {
      card_id: parseInt(this.props.match.params.card_id),
      topic_id: parseInt(this.props.match.params.topic_id),
      cardPreviewHTML: null
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
    const frontHTML = md.render(this.front.current.value);
    const backHTML = md.render(this.back.current.value);
    const cardHTML = `${frontHTML}<hr>${backHTML}`;
    this.setState({ cardPreviewHTML: ReactHtmlParser(cardHTML, options) });
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
    const frontHTML = md.render(frontMD);
    const backMD = this.back.current.value;
    const backHTML = md.render(backMD);
    const topicId = parseInt(document.getElementById("parent").value);

    if (this.state.card_id) {
      db.updateCard(
        this.state.card_id,
        frontMD,
        frontHTML,
        backMD,
        backHTML,
        topicId
      );
      this.goBack();
      return;
    }

    db.createCard(frontMD, frontHTML, backMD, backHTML, topicId);

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
    const card = db.getCard(this.state.card_id);

    return (
      <div className="card-editor-container">
        <div className="card-editor">
          <h4>Front</h4>
          <textarea
            id="front"
            ref={this.front}
            onInput={() => this.handleInput()}
            defaultValue={card ? card.front_md : null}
          />

          <h4>Back</h4>
          <textarea
            id="back"
            ref={this.back}
            onInput={() => this.handleInput()}
            defaultValue={card ? card.back_md : null}
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
          <div
            id="card-preview"
            className="card card-content"
            ref={this.preview}
          >
            {this.state.cardPreviewHTML}
          </div>
        </div>
      </div>
    );
  }
}
