import React from 'react';

import db from './../model/database';

import Card from './Card';

export default class Search extends React.Component {
    constructor(props) {
        super(props);
        this.renderCards = this.renderCards.bind(this);
    }

    renderCards() {
        const cards = db.getCardsByKeywords(this.props.match.params.text);
        const cardsList = cards.map(c =>
            <Card key={c.id} card={c} />
        );
        return <div className="cards-container">{cardsList}</div>;
    }

    render() {
        return (
            <div>
                <h2 className="text-center">Search</h2>
                <p className="text-center">Showing results for <i>{this.props.match.params.text}</i></p>
                {this.renderCards()}
            </div>
        );
    }
}
