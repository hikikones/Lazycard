import * as React from 'react';
import { Switch, Route, useRouteMatch, useParams, Redirect } from "react-router-dom";

import db, { Topic as TopicEntity } from '../model/Database';

import Layout, { Sidebar, SidebarItem, SidebarLink, Content } from './Layout';
import Topic from './Topic';
import Empty from './Empty';

const Topics = () => {
    const [topics, setTopics] = React.useState<TopicEntity[]>([...db.topics.getAll()]);
    const match = useRouteMatch();

    const onImport = () => {
        const success = db.import();
        if (success) updateTopics();
    }

    const onNewTopic = () => {
        db.topics.new("New Topic");
        updateTopics();
    }

    const updateTopics = () => {
        setTopics([...db.topics.getAll()]);
    }

    return (
        <Layout sidebarWidth={150}>

            <Sidebar>
                <SidebarItem name="Import" icon="save_alt" active={false} onClick={onImport} />
                <SidebarItem name="New topic" icon="add" active={false} onClick={onNewTopic} />
                <hr />
                {topics.map(t =>
                    <SidebarLink name={t.name} to={`${match.path}/${t.id}`} key={t.id} />
                )}
            </Sidebar>

            <Content>

                <div className="content">
                <Switch>
                    <Route exact path={match.path}>
                        {topics.length === 0
                            ?   <Empty icon="content_paste" message="No topics" />
                            :   <Redirect to={`${match.path}/${topics[0].id}`} />
                        }
                    </Route>
                    <Route path={`${match.path}/:topicId`}>
                        <TopicContainer onTopicChange={updateTopics} />
                    </Route>
                </Switch>
                </div>
            </Content>

        </Layout>
    );
}

const TopicContainer = (props: {onTopicChange(): void}) => {
    const { topicId } = useParams();
    const [topic, setTopic] = React.useState<TopicEntity>(db.topics.get(Number(topicId)));

    React.useEffect(() => {
        setTopic(db.topics.get(Number(topicId)));
    }, [topicId]);

    return <Topic topic={topic} onTopicChange={props.onTopicChange} />
}

export default Topics;