import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import CardMenu from './CardMenu';

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

    private edit = (card: CardEntity): void => {
        // TODO: show modal for card edit
        console.log("EDIT: " + card.id);
    }

    private delete = (card: CardEntity): void => {
        console.log("DELETE: " + card.id);
        db.cards.delete(card.id);
        this.props.onCardChange();
    }

    public render() {
        return (
            <div>
                <h2>Cards</h2>

                <button onClick={this.toggleAnswer}>Toggle Answer</button>

                {this.props.cards.map(c =>
                    <Card
                        key={c.id}
                        front={c.front}
                        back={this.state.showBack ? c.back : undefined}
                    >
                        <CardMenu card={c} onEdit={this.edit} onDelete={this.delete} />
                    </Card>
                )}
            </div>
        );
    }
}

interface IProps {
    cards: readonly CardEntity[]
    onCardChange(): void
}

interface IState {
    showBack: boolean
}