import * as React from 'react';
import { useParams } from "react-router-dom";

import db, { Card as CardEntity, Topic as TopicEntity } from '../model/Database';
import KeyCodes from '../controller/KeyCodes';

import Card from './Card';
import Button from './Button';
import Empty from './Empty';

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
    const { topicId } = useParams<{topicId: string}>();
    const [id] = React.useState<number>(Number(topicId));

    const cards = React.useRef<CardEntity[]>(shuffle(db.cards.getDue(id)));
    const index = React.useRef<number>(0);
    const [card, setCard] = React.useState<CardEntity>(cards.current[index.current]);
    const [showAnswer, setShowAnswer] = React.useState<boolean>(false);
    const [total, setTotal] = React.useState<number>(cards.current.length);
    const [enableShortcuts, setEnableShortcuts] = React.useState<boolean>(true);
    
    const customStudy = React.useRef<boolean>(false);

    const [topics] = React.useState<Record<number, TopicEntity>>(db.topics.getAllAsRecord());

    const handleReview = (success: boolean) => {
        if (!customStudy.current) card.review(success);
        removeCurrentCard();
        showNextCard();
    }

    const skip = () => {
        showNextCard();
    }

    const showNextCard = () => {
        index.current++;
        if (index.current >= cards.current.length) index.current = 0;
        showCard();
    }

    const showCard = () => {
        setShowAnswer(false);
        setCard(cards.current[index.current]);
    }

    const removeCurrentCard = () => {
        cards.current.splice(index.current, 1);
    }

    const toggleShortcuts = () => {
        setEnableShortcuts(enable => !enable);
    }

    const onDelete = () => {
        removeCurrentCard();
        setTotal(t => t - 1);
        showNextCard();
    }

    const initCustomStudy = () => {
        const cardsToStudy = id ? db.cards.getByTopic(id) : [...db.cards.getAll()];
        cards.current = shuffle(cardsToStudy);
        index.current = 0;
        customStudy.current = true;
        setTotal(cards.current.length);
        showCard();
    }

    if (cards.current.length === 0) {
        if ((id && db.cards.getByTopic(id).length === 0) || db.cards.size() === 0) {
            return (
                <main>
                    <Empty icon="content_copy" message="No cards" />
                </main>
            );
        }

        return (
            <main>
                <Empty icon="mood" message="No cards to review">
                    <Button icon="redo" name="Custom study" action={initCustomStudy} />
                </Empty>
            </main>
        );
    }

    return (
        <main className="col col-center space-between">
            <section>
                <label>{total - cards.current.length} / {total}</label>
            </section>

            <Card
                card={card}
                showBack={showAnswer}
                onDelete={onDelete}
                onToggleModal={toggleShortcuts}
            >
                {/* <label className="card-topic-name">
                    {topics[card.topicId].name}
                </label> */}
            </Card>

            {enableShortcuts &&
                <div className="review-buttons space-fixed">
                    {showAnswer || <Button icon="lock_open" action={() => setShowAnswer(true)} shortcut={KeyCodes.Space} />}
                    {showAnswer && <Button icon="done" action={() => handleReview(true)} shortcut={KeyCodes.ArrowUp} />}
                    {showAnswer && <Button icon="close" action={() => handleReview(false)} shortcut={KeyCodes.ArrowDown} />}
                    <Button icon="double_arrow" action={skip} shortcut={KeyCodes.ArrowRight} />
                </div>
            }
            {enableShortcuts ||
                <div className="review-buttons space-fixed">
                    {showAnswer || <Button icon="lock_open" action={() => setShowAnswer(true)} />}
                    {showAnswer && <Button icon="done" action={() => handleReview(true)} />}
                    {showAnswer && <Button icon="close" action={() => handleReview(false)} />}
                    <Button icon="double_arrow" action={skip} />
                </div>
            }
        </main>
    );
}

export default Review;