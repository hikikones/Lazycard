import * as React from 'react';

import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';

export default class CardSelectable extends React.Component<ICard, IState> {
    public constructor(props: ICard) {
        super(props);
        this.state = {
            showCheckbox: this.props.card.selected
        }
    }

    private showCheckbox = (): void => {
        this.setState({ showCheckbox: true });
    }

    private hideCheckbox = (): void => {
        this.setState({ showCheckbox: false });
    }

    private toggleSelect = (): void => {
        this.props.card.selected = !this.props.card.selected;
        if (this.props.card.selected) this.props.onSelect();
        else this.props.onDeselect();
    }

    public render() {
        return (
            <div onMouseEnter={this.showCheckbox} onMouseLeave={this.hideCheckbox}>
                {this.props.card.selected || this.state.showCheckbox
                    ?   <div className="card-checkbox-container">
                            <Button
                                name=""
                                icon={this.props.card.selected ? "check_box" : "check_box_outline_blank"}
                                action={this.toggleSelect}
                            />
                        </div>
                    :   null
                }
                <Card
                    card={this.props.card}
                    showBack={this.props.showBack}
                    onDelete={() => this.props.onDelete(this.props.card)}
                />
            </div>
        );
    }
}

interface ICard {
    card: CardEntity
    showBack: boolean
    onDelete(card: CardEntity): void
    onSelect(): void
    onDeselect(): void
}

interface IState {
    showCheckbox: boolean
}