import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';
import Card from './Card';

export default class Review extends React.Component<IProps, IState> {
    private cards: CardEntity[];

    public constructor(props: IProps) {
        super(props);
        const topicId = Number(this.props.match.params.topicId);
        const cardsToReview = Number.isNaN(topicId) ? db.cards.getAll() : db.cards.getByTopic(topicId);
        this.cards = [...cardsToReview];

        this.shuffle();

        this.state = {
            currentCard: this.cards.pop(),
            showAnswer: false
        }
    }

    private showAnswer = (): void => {
        this.setState({ showAnswer: true });
    }

    private showNextCard = (): void => {
        this.setState({ showAnswer: false });
        this.setState({ currentCard: this.cards.pop() });
    }

    private skipCard = (): void => {
        this.cards.unshift(this.state.currentCard);
        this.showNextCard();
    }

    private handleReview = (correct: boolean): void => {
        // TODO
        console.log("ANSWER: " + correct);
        this.showNextCard();
    }

    private noCardsLeft = (): boolean => {
        return this.state.currentCard === undefined;
    }

    private showReviewButtons = (): JSX.Element => {
        return (
            <footer>
                <button onClick={() => this.handleReview(true)}>Yes</button>
                <button onClick={() => this.handleReview(false)}>No</button>
            </footer>
        )
    }

    private shuffle = (): void => {
        for (let currentIndex = this.cards.length - 1; currentIndex > 0; currentIndex--) {
            const newIndex = Math.floor(Math.random() * (currentIndex + 1));
            const temp = this.cards[currentIndex];
            this.cards[currentIndex] = this.cards[newIndex];
            this.cards[newIndex] = temp;
        }
    }

    public render() {
        if (this.noCardsLeft()) {
            return <h4>No more cards left!</h4>
        }

        return (
            <div>
                <h2>Review</h2>

                <Card
                    front={this.state.currentCard.front}
                    back={this.state.showAnswer ? this.state.currentCard.back : null}
                />

                {this.state.showAnswer ? null : <button onClick={this.showAnswer}>Show answer</button>}

                {this.state.showAnswer ? this.showReviewButtons() : null}
                <button onClick={this.skipCard}>Skip</button>
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