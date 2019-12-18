import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import CardMenu from './CardMenu';
import Modal from './Modal';
import CardEditor from './CardEditor';

export default class Cards extends React.Component<IProps, IState> {
    private selectedCard: CardEntity;

    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false,
            showModal: false
        }
    }

    private toggleAnswer = (): void => {
        this.setState({ showBack: !this.state.showBack });
    }

    private edit = (card: CardEntity): void => {
        this.selectedCard = card;
        this.setState({ showModal: true });
    }

    private delete = (card: CardEntity): void => {
        db.cards.delete(card.id);
        this.props.onCardChange();
    }

    private closeModal = () => {
        this.setState({ showModal: false });
    }

    private onCardEdit = () => {
        this.props.onCardChange();
        this.closeModal();
    }

    public render() {
        return (
            <div>
                <h2>Cards</h2>

                <section>
                    <button onClick={this.toggleAnswer}>Toggle Answer</button>
                </section>

                <div className="cards">
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

                <Modal show={this.state.showModal} onClickOutside={this.closeModal}>
                    <CardEditor
                        onSave={this.onCardEdit}
                        onCancel={this.closeModal}
                        card={this.selectedCard}
                    />
                </Modal>
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
    showModal: boolean
}