import * as React from 'react';

import db, { Card } from '../model/Database';

import Topic from './Topic';

// TODO: Generalise sidebar|content layout so other pages can use it
// TODO: Add Import to sidebar
// TODO: Add New Topic to sidebar

const Topics = () => {
    const [id, setId] = React.useState<number>(1);
    const [cards, setCards] = React.useState<Card[]>(db.cards.getByTopic(id));

    React.useEffect(() => {
        updateCards();
    }, [id]);

    const updateCards = () => {
        setCards(db.cards.getByTopic(id));
    }

    const updateTopics = () => {
        console.log("TODO: onTopicChange... update sidebar");
    }

    return (
        <div className="topics">

            <div className="topics-sidebar">
                {db.topics.getAll().map(t =>
                    <TopicLink name={t.name} icon="bookmark" active={t.id === id} onClick={() => setId(t.id)} key={t.id} />
                )}
            </div>

            <div className="topics-content content">
                <Topic
                    topic={db.topics.get(id)}
                    cards={cards}
                    onTopicChange={updateTopics}
                    onCardChange={updateCards}
                />
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
                <h3>content.....</h3>
            </div>

        </div>
    );
}

const TopicLink = (props: ITopicLinkProps) => {
    const classNames = props.active ? "topic-nav active" : "topic-nav";
    return (
        <a onClick={props.onClick} className={classNames}>
            <i className="material-icons">{props.icon}</i> {props.name}
        </a>
    );
}

interface ITopicLinkProps {
    name: string
    icon: string
    active: boolean
    onClick(): void
}

export default Topics;