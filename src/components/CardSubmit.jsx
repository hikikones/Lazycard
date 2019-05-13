import React from 'react';
import db from './../model/database';

import TopicsSelect from './TopicsSelect';

const md = require('markdown-it')();
const prism = require('markdown-it-prism');
const mk = require('@iktakahiro/markdown-it-katex');

md.use(prism);
md.use(mk);

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
    this.preview.current.innerHTML = cardHTML;
  }

  resizeTextarea() {
    this.front.current.style.height = 'inherit';
    this.front.current.style.height = `${this.front.current.scrollHeight}px`;
    this.back.current.style.height = 'inherit';
    this.back.current.style.height = `${this.back.current.scrollHeight}px`;
  }

  clearInput() {
    this.front.current.value = '';
    this.back.current.value = '';
  }

  save() {
    // this.goBack();
    // console.log(this.state);
    const frontMD = this.front.current.value;
    const frontHTML = md.render(frontMD);
    const backMD = this.back.current.value;
    const backHTML = md.render(backMD);
    const topicId = parseInt(document.getElementById('parent').value);

    if (this.state.card_id) {
      db.updateCard(this.state.card_id, frontMD, frontHTML, backMD, backHTML, topicId);
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

          <TopicsSelect card_id={this.state.card_id} topic_id={this.state.topic_id} />

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
          <div id="card-preview" className="card card-content" ref={this.preview} />
        </div>
      </div>
    );
  }
}

/*
import React from 'react';
// import Prism from 'prismjs';
import db from './../model/database';

import TopicsSelect from './TopicsSelect';

const md = require('markdown-it')();
const prism = require('markdown-it-prism');
const mk = require('@iktakahiro/markdown-it-katex');

md.use(prism);
md.use(mk);

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
    };
  }

  save() {
    // this.goBack();
    // console.log(this.state);
    const frontMD = this.front.current.innerText;
    const frontHTML = md.render(this.front.current.innerText);
    const backMD = this.back.current.innerText;
    const backHTML = md.render(this.back.current.innerText);
    const topicId = parseInt(document.getElementById('parent').value);

    db.createCard(frontMD, frontHTML, backMD, backHTML, topicId);

    this.clearInput();
    this.renderMarkdown();
  }

  cancel() {
    this.goBack();
  }

  goBack() {
    this.props.history.goBack();
  }

  pasteAsPlainText(event) {
    event.preventDefault();
    const text = event.clipboardData.getData('text/plain');
    document.execCommand('insertHTML', false, text);
  }

  renderMarkdown() {
    const frontHTML = md.render(this.front.current.innerText);
    const backHTML = md.render(this.back.current.innerText);
    const cardHTML = `${frontHTML}<hr>${backHTML}`;
    this.preview.current.innerHTML = cardHTML;
  }

  clearInput() {
    this.front.current.innerHTML = '';
    this.back.current.innerHTML = '';
  }

  resizeBox(event) {
    console.log(event.target);
    event.target.style.height = `${event.target.scrollHeight}px`;
  }

  render(props) {
    return (
      <div className="card-editor-container">
        <div className="card-editor">
          <h4>Front</h4>
          <div
            id="front"
            contentEditable="true"
            className="markdown-box"
            ref={this.front}
            onPaste={event => this.pasteAsPlainText(event)}
            onKeyUp={() => this.renderMarkdown()}
          />

          <h4>Back</h4>
          <div
            id="back"
            contentEditable="true"
            className="markdown-box"
            ref={this.back}
            onPaste={event => this.pasteAsPlainText(event)}
            onKeyUp={() => this.renderMarkdown()}
          />

          <textarea name="text" onInput={event => this.resizeBox(event)} />

          <TopicsSelect card_id={this.state.card_id} topic_id={this.state.topic_id} />

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
          <div id="card-preview" className="card card-content" ref={this.preview}>
            <p>
              oipgjre oij oi j oj oij er jore j oij ioj j oi jiorgej oierg jeirgjgeroi j regojrg{' '}
            </p>
            <hr />
            <p>oiwejf oi j weij weoij fje oin nifewionionon ui ewhioehj wne new</p>
          </div>
        </div>
      </div>
    );
  }
}
*/
