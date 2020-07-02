import * as React from 'react';

import db, { Card as CardEntity, Topic as TopicEntity } from '../model/Database';

import Layout, { Sidebar, SidebarItem, Content } from './Layout';
import Topic from './Topic';

// TODO: Display something nice when no topics

const Topics = () => {
    const [topics, setTopics] = React.useState<TopicEntity[]>([...db.topics.getAll()]);
    const [topic, setTopic] = React.useState<TopicEntity>(topics[0]);
    const [cards, setCards] = React.useState<CardEntity[]>(topic === undefined ? [] : db.cards.getByTopic(topic.id));

    React.useEffect(() => {
        updateCards();
    }, [topic]);

    const updateCards = () => {
        if (topic === undefined) return;
        setCards(db.cards.getByTopic(topic.id));
    }

    const updateTopics = () => {
        setTopics([...db.topics.getAll()]);
    }

    const newTopic = () => {
        db.topics.new("New Topic");
        if (topic === undefined) setTopic(db.topics.getAll()[0]);
        updateTopics();
    }

    return (
        <Layout sidebarWidth={150}>

            <Sidebar>
                <SidebarItem name="Import" icon="save_alt" active={false} onClick={() => db.import()} />
                <SidebarItem name="New topic" icon="add" active={false} onClick={newTopic} />
                <hr />
                {topics.map(t =>
                    <SidebarItem name={t.name} active={t.id === topic.id} onClick={() => setTopic(db.topics.get(t.id))} key={t.id} />
                )}
            </Sidebar>

            <Content>
                {topic === undefined
                    ?   <div className="content">
                            <h2>No topics...</h2>
                        </div>
                    :   <Topic
                            topic={topic}
                            cards={cards}
                            onTopicChange={updateTopics}
                            onCardChange={updateCards}
                        />
                }
            </Content>

        </Layout>
    );
}

export default Topics;