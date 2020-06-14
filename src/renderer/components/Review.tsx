import * as React from 'react';

import db, { Card } from '../model/Database';

const shuffle = (arr: Card[]): Card[] => {
    for (let currentIndex = arr.length - 1; currentIndex > 0; currentIndex--) {
        const newIndex = Math.floor(Math.random() * (currentIndex + 1));
        const temp = arr[currentIndex];
        arr[currentIndex] = arr[newIndex];
        arr[newIndex] = temp;
    }
    return arr;
}

const Review = () => {
    const [cards, setCards] = React.useState<Array<Card>>(shuffle(db.cards.getDue()));
    const [index, setIndex] = React.useState<number>(0);
    const [card, setCard] = React.useState<Card>(cards[index]);
    const [showAnswer, setShowAnswer] = React.useState<boolean>(false);
    
    React.useEffect(() => {
        if (index > cards.length - 1) {
            shuffle(cards);
            setIndex(0);
        }
        showCard();
    }, [index, cards]);

    const handleReview = (success: boolean) => {
        card.review(success);
        setCards(cards.filter(c => c.id !== card.id));
    }

    const skip = () => {
        setIndex(index + 1);
        showCard();
    }

    const showCard = () => {
        setShowAnswer(false);
        setCard(cards[index]);
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
        <div className="content review">
            <h1>Review</h1>
            <img src={showAnswer ? card.back : card.front} />
            <div className="review-buttons">
                {showAnswer || <ReviewButton icon="lock_open" action={() => setShowAnswer(true)} />}
                {showAnswer && <ReviewButton icon="done" action={() => handleReview(true)} />}
                {showAnswer && <ReviewButton icon="close" action={() => handleReview(false)} />}
                <ReviewButton icon="double_arrow" action={() => skip()} />
            </div>
        </div>
    );
}

const ReviewButton = (props: {icon: string, action(): void}) => {
    return (
        <a onClick={props.action}>
                <i className="material-icons">{props.icon}</i>
        </a>
    );
}

export default Review;