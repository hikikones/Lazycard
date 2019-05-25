import React from "react";
import { Link } from "react-router-dom";
import path from "path";

import db from "./../model/database";
import Config from "./../controller/Config";

const md = require("markdown-it")({
  modifyToken: function(token, env) {
    switch (token.type) {
      case "image":
        token.attrObj.src = path.join(
          Config.getImagesPath(),
          token.attrObj.src
        );
        break;
    }
  }
});
const prism = require("markdown-it-prism");
const mk = require("@iktakahiro/markdown-it-katex");
const tk = require("markdown-it-modify-token");

md.use(prism);
md.use(mk);
md.use(tk);

export default class Card extends React.Component {
  constructor(props) {
    super(props);

    this.renderFront = this.renderFront.bind(this);
    this.renderBack = this.renderBack.bind(this);
    this.renderButtons = this.renderButtons.bind(this);
    this.toggleDropdown = this.toggleDropdown.bind(this);
    this.delete = this.delete.bind(this);

    this.state = {
      renderCard: true,
      showDropdown: false,
    };
  }

  toggleDropdown() {
    this.setState({ showDropdown: !this.state.showDropdown });
  }

  delete() {
    db.deleteCard(this.props.card.id);
    this.setState({ renderCard: false });
  }

  renderFront() {
    const renderFront = typeof this.props.renderFront === "undefined"
      ? true
      : this.props.renderFront;
    if (!renderFront) {
      return null;
    }
    const frontHTML = md.render(this.props.card.front);
    return <div dangerouslySetInnerHTML={{__html: frontHTML}} />
  }

  renderBack() {
    const renderBack = typeof this.props.renderBack === "undefined"
      ? true
      : this.props.renderBack;
    if (!renderBack) {
      return null;
    }
    const backHTML = md.render(this.props.card.back);
    return (
      <div>
        <hr/>
        <div dangerouslySetInnerHTML={{__html: backHTML}} />
      </div>
    );
  }

  renderButtons() {
    const renderButtons = typeof this.props.renderButtons === "undefined"
      ? true
      : this.props.renderButtons;
    if (!renderButtons) {
      return null;
    }
    return (
      <div>
        <Link
          to={`/card/submit/${this.props.card.id}/${this.props.card.topic_id}`}
          className="button"
        >
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
            onClick={this.props.deleteAction ? this.props.deleteAction : this.delete}
          />
        </div>
      </div>
    );
  }

  render() {
    if (!this.state.renderCard) {
      return null;
    }

    return (
      <div className="card card-content">
        {this.renderFront()}
        {this.renderBack()}
        {this.renderButtons()}
        {this.props.children}
      </div>
    );
  }
}
