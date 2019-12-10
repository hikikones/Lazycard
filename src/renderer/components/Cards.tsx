import * as React from 'react';

import db from '../model/Database';

import Card from './Card';

export default class Cards extends React.Component<IProps> {
    private cards = this.props.topicId ? db.cards.getByTopic(this.props.topicId) : db.cards.getAll();

    public constructor(props: IProps) {
        super(props);
    }

    public render() {
        return (
            <div>
                <h2>Cards</h2>
                {this.cards.map(c => <Card key={c.id} front={c.front} back={c.back} />)}
            </div>
        );
    }
}

interface IProps {
    topicId?: number
}