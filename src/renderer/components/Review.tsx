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
    private readonly topicId: number | undefined;
    private cards: CardEntity[];
    private total: number;
    private customStudy: boolean = false;

    private unlockBtn = React.createRef<Button>();
    private yesBtn = React.createRef<Button>();
    private noBtn = React.createRef<Button>();
    private skipBtn = React.createRef<Button>();

    public constructor(props: IProps) {
        super(props);

        const topicId = Number(this.props.match.params.topicId);
        this.topicId = Number.isNaN(topicId) ? undefined : topicId;

        this.init();

        this.state = {
            currentCard: this.cards.pop(),
            showAnswer: false,
            showModal: false
        };

        this.initKeyboardShortcuts();
    }

    private init = (): void => {
        this.cards = this.fetchCards();
        this.total = this.cards.length;
        this.shuffle();
    }

    private fetchCards = (): CardEntity[] => {
        if (this.customStudy) {
            return this.topicId === undefined
                ?   [...db.cards.getAll()]
                :   db.cards.getByTopic(this.topicId);
        }
        return this.topicId === undefined ? db.cards.getDue() : db.cards.getDue(this.topicId);
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
        if (!this.customStudy) {
            srs.schedule(this.state.currentCard, correct);
        }
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

    private initCustomStudy = (): void => {
        this.customStudy = true;
        this.init();
        this.showNextCard();
    }

    private initKeyboardShortcuts = (): void => {
        document.onkeydown = (e: KeyboardEvent) => {
            // TODO: remove global event when leaving page
            switch (e.keyCode) {
                case 32: // Space
                    if (this.unlockBtn.current) this.unlockBtn.current.click();
                    break;
                case 38: // ArrowUp
                    if (this.yesBtn.current) this.yesBtn.current.click();
                    break;
                case 39: // ArrowRight
                    if (this.skipBtn.current) this.skipBtn.current.click();
                    break;
                case 40: // ArrowDown
                    if (this.noBtn.current) this.noBtn.current.click();
                    break;
            }
        };
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
            return (
                <div>
                    <h2>Good job!</h2>
                    <p>There are no more remaining cards to be reviewed.</p>
                    <p>Do a custom study for reviewing as many cards as you like without affecting the scheduler.</p>
                    <Button name="Custom study" icon="redo" action={this.initCustomStudy} />
                </div>
            );
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
                    {this.state.showAnswer ? null : <Button ref={this.unlockBtn} name="" icon="lock_open" action={this.showAnswer} />}
                </section>

                <section>
                    {this.state.showAnswer ? <Button ref={this.yesBtn} name="" icon="done" action={() => this.handleReview(true)} /> : null}
                    {this.state.showAnswer ? <Button ref={this.noBtn} name="" icon="close" action={() => this.handleReview(false)} /> : null}
                    <Button ref={this.skipBtn} name="" icon="double_arrow" action={this.skipCard} />
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