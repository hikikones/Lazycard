import * as React from 'react';
import { Switch, Route, useRouteMatch, useParams, Redirect } from "react-router-dom";

import db, { Topic as TopicEntity } from '../model/Database';

import Layout, { Sidebar, SidebarItem, SidebarLink, Content } from './Layout';
import Topic from './Topic';
import Modal from './Modal';
import Empty from './Empty';
import Button from './Button';

// TODO: update cards on import if currently in same topic

const Topics = () => {
    const [topics, setTopics] = React.useState<TopicEntity[]>([...db.topics.getAll()]);
    const match = useRouteMatch();

    const [showImportOptions, setShowImportOptions] = React.useState<boolean>(false);

    const toggleImportOptions = () => {
        setShowImportOptions(show => !show);
    }

    const onImport = (mergeTopic: boolean, allowDuplicateCards: boolean) => {
        const success = db.import(mergeTopic, allowDuplicateCards);
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
                <SidebarItem name="Import" icon="save_alt" active={false} onClick={toggleImportOptions} />
                <SidebarItem name="New topic" icon="add" active={false} onClick={onNewTopic} />
                <hr />
                {topics.map(t =>
                    <SidebarLink name={t.name} to={`${match.path}/${t.id}`} key={t.id} />
                )}
            </Sidebar>

            <Content>
                <div className="content">
                    <ImportOptions
                        show={showImportOptions}
                        onClickOutside={toggleImportOptions}
                        onImport={onImport}
                    />
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

const ImportOptions = (props: IImportOptionsProps) => {
    const [mergeTopic, setMergeTopic] = React.useState<boolean>(false);
    const [allowDuplicateCards, setAllowDuplicateCards] = React.useState<boolean>(false);

    const onImport = () => {
        props.onClickOutside();
        props.onImport(mergeTopic, allowDuplicateCards);
    }
    
    return (
        <Modal show={props.show} onClickOutside={props.onClickOutside} >
            <h2>Import</h2>
            <table>
                <thead>
                    <tr>
                        <th>Options</th>
                        <th>Description</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>
                            <Button
                                name="Merge Topic"
                                icon={mergeTopic ? "check_box" : "check_box_outline_blank"}
                                action={() => setMergeTopic(ok => !ok)}
                            />
                        </td>
                        <td>
                            Import cards into an existing topic with the same name.
                            If no such topic was found, a new one will be created.
                            When deselected, a new topic will always be created.
                        </td>
                    </tr>
                    <tr>
                        <td>
                            <Button
                                name="Duplicate Cards"
                                icon={allowDuplicateCards ? "check_box" : "check_box_outline_blank"}
                                action={() => setAllowDuplicateCards(ok => !ok)}
                            />
                        </td>
                        <td>
                            Allow duplicate cards to be imported.
                            When selected, all cards will be imported regardless
                            if they already exist or not.
                        </td>
                    </tr>
                </tbody>
            </table>
            <section className="row space-between">
                <Button name="Import" icon="save_alt" action={onImport} />
                <Button name="Cancel" icon="close" action={props.onClickOutside} />
            </section>
        </Modal>
    );
}

interface IImportOptionsProps {
    show: boolean
    onClickOutside(): void
    onImport(mergeTopic: boolean, allowDuplicateCards: boolean): void
}

export default Topics;