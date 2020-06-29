import * as React from 'react';

import db, { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';

// TODO: rework?
// TODO: custom study

const shuffle = (arr: CardEntity[]): CardEntity[] => {
    for (let currentIndex = arr.length - 1; currentIndex > 0; currentIndex--) {
        const newIndex = Math.floor(Math.random() * (currentIndex + 1));
        const temp = arr[currentIndex];
        arr[currentIndex] = arr[newIndex];
        arr[newIndex] = temp;
    }
    return arr;
}

const Review = () => {
    const [cards, setCards] = React.useState<CardEntity[]>(shuffle(db.cards.getDue()));
    const [index, setIndex] = React.useState<number>(0);
    const [card, setCard] = React.useState<CardEntity>(cards[index]);
    const [showAnswer, setShowAnswer] = React.useState<boolean>(false);
    
    React.useLayoutEffect(() => {
        if (index > cards.length - 1) {
            //shuffle(cards);
            setIndex(0);
        }
        showCard();
    }, [index, cards]);

    const handleReview = (success: boolean) => {
        card.review(success);
        setCards(oldCards => oldCards.filter(c => c.id !== card.id));
    }

    const skip = () => {
        setIndex(i => i + 1);
        showCard();
    }

    const showCard = () => {
        setShowAnswer(false);
        setCard(cards[index]);
    }

    const onDelete = () => {
        // this.total--;
        // this.showNextCard();
        console.log("TODO: onDelete from review");
    }

    if (db.cards.size() === 0) {
        return (
            <div className="content">
                <h2>No cards...</h2>
            </div>
        );
    }

    if (card === undefined) {
        return (
            <div className="content">
                <h2>Good job!</h2>
            </div>
        );
    }

    return (
        <div className="content col col-center space-between full-height">
            <h1>Review</h1>

            <Card
                card={card}
                showBack={showAnswer}
                onDelete={onDelete}
            />

            <div className="review-buttons space-fixed">
                {showAnswer || <Button icon="lock_open" action={() => setShowAnswer(true)} />}
                {showAnswer && <Button icon="done" action={() => handleReview(true)} />}
                {showAnswer && <Button icon="close" action={() => handleReview(false)} />}
                <Button icon="double_arrow" action={() => skip()} />
            </div>
        </div>
    );
}

export default Review;