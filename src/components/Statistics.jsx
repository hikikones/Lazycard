import React from "react";
import { Link } from "react-router-dom";

import db from "./../model/database";

export default class Statistics extends React.Component {
  getTotalCards() {
    if (this.props.topicId === 0) {
        return db.getAllCardsLength();
    }
    return db.getCardsRecursivelyLength(this.props.topicId);
  }

  getDueCards() {
      if (this.props.topicId === 0) {
          return db.getAllDueCardsLength();
      }
      return db.getDueCardsLength(this.props.topicId);
  }

  render() {
    return (
      <div className="stats">
        <table className="text-center">
            <thead>
                <tr>
                    <th>Total cards</th>
                    <th>Due cards</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td className="text-center">{this.getTotalCards()}</td>
                    <td className="text-center">{this.getDueCards()}</td>
                </tr>
            </tbody>
        </table>
        <Link to={`/topic/review/${this.props.topicId}`} className="button">
        REVIEW
        </Link>
      </div>
    );
  }
}
