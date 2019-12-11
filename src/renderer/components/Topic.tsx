import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card } from '../model/Database';

import Cards from './Cards';

export default class Topic extends React.Component<IProps, IState> {
    private readonly topic = db.topics.get(parseInt(this.props.match.params.id));

    public constructor(props: IProps) {
        super(props);
        this.state = {
            name: this.topic.name,
            cards: db.cards.getByTopic(this.topic.id)
        }
    }

    private changeName = (newName: string): void => {
        this.topic.name = newName;
        this.setState({ name: newName });
        this.props.onTopicChange();
    }

    private newCard = () => {
        const card: Card = new Card(this.topic.id);
        card.front = "Front";
        card.back = "Back";
        db.cards.add(card);
        this.setState({ cards: db.cards.getByTopic(this.topic.id) });
    }

    public render() {
        return (
            <div>
                <h1>{this.state.name}</h1>
                <button onClick={() => this.changeName("New Name")}>Change name</button>

                <button onClick={() => this.newCard()}>New card</button>

                <Cards cards={this.state.cards} />
            </div>
        );
    }
}

interface IProps extends RouteComponentProps<{ id: string }> {
    onTopicChange(): void
}

interface IState {
    name: string
    cards: readonly Card[]
}