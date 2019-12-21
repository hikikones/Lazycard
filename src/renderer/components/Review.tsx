import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';
import srs from '../controller/SRS';

import Card from './Card';
import Button from './Button';
import CardMenu from './CardMenu';
import Modal from './Modal';
import CardEditor from './CardEditor';

export default class Review extends React.Component<IProps, IState> {
    private cards: CardEntity[];
    private total: number;

    public constructor(props: IProps) {
        super(props);

        const topicId = Number(this.props.match.params.topicId);
        this.cards = Number.isNaN(topicId) ? db.cards.getDue() : db.cards.getDue(topicId);
        this.total = this.cards.length;

        this.shuffle();

        this.state = {
            currentCard: this.cards.pop(),
            showAnswer: false,
            showModal: false
        }
    }

    private showAnswer = (): void => {
        this.setState({ showAnswer: true });
    }

    private showNextCard = (): void => {
        this.setState({ showAnswer: false, currentCard: this.cards.pop() });
    }

    private skipCard = (): void => {
        this.cards.unshift(this.state.currentCard);
        this.showNextCard();
    }

    private handleReview = (correct: boolean): void => {
        srs.schedule(this.state.currentCard, correct);
        this.showNextCard();
    }

    private noCardsLeft = (): boolean => {
        return this.state.currentCard === undefined;
    }

    private shuffle = (): void => {
        for (let currentIndex = this.cards.length - 1; currentIndex > 0; currentIndex--) {
            const newIndex = Math.floor(Math.random() * (currentIndex + 1));
            const temp = this.cards[currentIndex];
            this.cards[currentIndex] = this.cards[newIndex];
            this.cards[newIndex] = temp;
        }
    }

    private edit = (card: CardEntity): void => {
        this.setState({ showModal: true });
    }

    private delete = (card: CardEntity): void => {
        db.cards.delete(card.id);
        this.total--;
        this.showNextCard();
    }

    private closeModal = (): void => {
        this.setState({ showModal: false });
    }

    public render() {
        if (this.noCardsLeft()) {
            return <h3>Good job!</h3>
        }

        return (
            <div className="col col-center review space-between">
                <span className="review-progress">{this.total - this.cards.length} / {this.total}</span>

                <section className="col col-center review-card">
                    <Card
                        front={this.state.currentCard.front}
                        back={this.state.showAnswer ? this.state.currentCard.back : undefined}
                    >
                        <CardMenu card={this.state.currentCard} onEdit={this.edit} onDelete={this.delete} />
                    </Card>
                    {this.state.showAnswer ? null : <Button name="" icon="lock_open" action={this.showAnswer} />}
                </section>

                <section>
                    {this.state.showAnswer ? <Button name="" icon="done" action={() => this.handleReview(true)} /> : null}
                    {this.state.showAnswer ? <Button name="" icon="close" action={() => this.handleReview(false)} /> : null}
                    <Button name="" icon="double_arrow" action={this.skipCard} />
                </section>

                <Modal show={this.state.showModal} onClickOutside={this.closeModal}>
                    <CardEditor
                        onSave={this.closeModal}
                        onCancel={this.closeModal}
                        card={this.state.currentCard}
                    />
                </Modal>
            </div>
        );
    }
}

interface IProps extends RouteComponentProps<{ topicId?: string }> {
}

interface IState {
    currentCard: CardEntity
    showAnswer: boolean
    showModal: boolean
}