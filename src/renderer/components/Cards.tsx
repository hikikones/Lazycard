import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import Card from './Card';

export default class Cards extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false
        }
    }

    private toggleAnswer = (): void => {
        this.setState({ showBack: !this.state.showBack });
    }

    public render() {
        const cards = this.props.cards ? this.props.cards : db.cards.getAll();

        return (
            <div>
                <h2>Cards</h2>

                <button onClick={this.toggleAnswer}>Toggle Answer</button>

                {cards.map(c =>
                    <Card
                        key={c.id}
                        front={c.front}
                        back={this.state.showBack ? c.back : null}
                    />
                )}
            </div>
        );
    }
}

interface IProps {
    cards?: readonly CardEntity[]
}

interface IState {
    showBack: boolean
}