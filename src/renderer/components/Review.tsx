import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card as CardEntity, Topic } from '../model/Database';

import Card from './Card';
import Button from './Button';

export default class Review extends React.Component<IProps, IState> {
    private readonly topic: Topic | undefined;
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
        this.topic = Number.isNaN(topicId) ? undefined : db.topics.get(topicId);

        this.init();

        this.state = {
            currentCard: this.cards.pop(),
            showAnswer: false
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
            return this.topic === undefined
                ?   [...db.cards.getAll()]
                :   db.cards.getByTopic(this.topic.id);
        }
        return this.topic === undefined ? db.cards.getDue() : db.cards.getDue(this.topic.id);
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

    private handleReview = (success: boolean): void => {
        if (!this.customStudy) {
            this.state.currentCard.review(success);
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

    private onDelete = (): void => {
        this.total--;
        this.showNextCard();
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
                <section className="review-progress">
                    {this.topic === undefined ? null : <label>{this.topic.name}</label>}
                    <span>{this.total - this.cards.length} / {this.total}</span>
                </section>

                <section className="col col-center review-card">
                    <Card
                        card={this.state.currentCard}
                        showBack={this.state.showAnswer}
                        onDelete={this.onDelete}
                    />
                    {this.state.showAnswer ? null : <Button ref={this.unlockBtn} name="" icon="lock_open" action={this.showAnswer} />}
                </section>

                <section>
                    {this.state.showAnswer ? <Button ref={this.yesBtn} name="" icon="done" action={() => this.handleReview(true)} /> : null}
                    {this.state.showAnswer ? <Button ref={this.noBtn} name="" icon="close" action={() => this.handleReview(false)} /> : null}
                    <Button ref={this.skipBtn} name="" icon="double_arrow" action={this.skipCard} />
                </section>
            </div>
        );
    }
}

interface IProps extends RouteComponentProps<{ topicId?: string }> {
}

interface IState {
    currentCard: CardEntity
    showAnswer: boolean
}