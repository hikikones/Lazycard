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
            <div>
                <h2>No cards...</h2>
            </div>
        );
    }

    if (card === undefined) {
        return (
            <div>
                <h2>Good job!</h2>
            </div>
        );
    }

    return (
        <div>
            <h1>Review</h1>
            <img src={showAnswer ? card.back : card.front} />
            <div>
                {showAnswer ? null : <button onClick={() => setShowAnswer(true)}>Show</button>}
                <button onClick={() => handleReview(true)}>Yes</button>
                <button onClick={() => handleReview(false)}>No</button>
                <button onClick={() => skip()}>Skip</button>
                <button onClick={() => console.log("COUNT:" + index)}>COUNT</button>
            </div>
        </div>
    );
}

export default Review;