import * as React from 'react';

import db, { Card } from '../model/Database';

import Layout, { Sidebar, SidebarItem, Content } from './Layout';
import Topic from './Topic';

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
        <Layout sidebarWidth={150}>

            <Sidebar>
                <SidebarItem name="Import" icon="save_alt" active={false} onClick={() => db.import()} />

                <hr />

                {db.topics.getAll().map(t =>
                    <SidebarItem name={t.name} active={t.id === id} onClick={() => setId(t.id)} key={t.id} />
                )}
            </Sidebar>

            <Content>
                <Topic
                    topic={db.topics.get(id)}
                    cards={cards}
                    onTopicChange={updateTopics}
                    onCardChange={updateCards}
                />
            </Content>

        </Layout>
    );
}

export default Topics;