import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity, Topic as TopicEntity } from '../model/Database';

import Cards from './Cards';
import CardEditor from './CardEditor';
import Button from './Button';
import ButtonLink from './ButtonLink';
import Dropdown, { DropdownItem } from './Dropdown';

// TODO: Make <EditableHeader />

const Topic = (props: ITopicProps) => {
    const [showCardEditor, setShowCardEditor] = React.useState<boolean>(false);

    const toggleCardEditor = () => {
        setShowCardEditor(show => !show);
    }

    const onDelete = () => {
        // this.setState({ deleted: true });
        // db.topics.delete(this.topic.id);
        // this.props.onTopicChange();
        // this.state.cards.forEach(c => db.cards.delete(c.id));

        console.log("TODO: delete topic and its cards");
        props.onTopicChange();
    }

    return (
        <div className="content">
            <h1>{props.topic.name}</h1>

            <section>
                {showCardEditor || <Button name="Add new card" icon="add" action={toggleCardEditor} />}
                <ButtonLink name="Review" icon="drafts" to={`/review/${props.topic.id}`} />
                <Dropdown name="Export" icon="save" showDownArrow={true}>
                    <DropdownItem name="JSON" icon="archive" action={() => db.export(props.topic.id)} />
                    <DropdownItem name="HTML" icon="file_copy" action={() => db.exportToHTML(props.topic.id)} />
                </Dropdown>
                <Button name="Delete" icon="delete" action={onDelete} />
            </section>

            {showCardEditor && <CardEditor topicId={props.topic.id} onSave={props.onCardChange} onCancel={toggleCardEditor} />}

            <Cards cards={props.cards} onCardChange={props.onCardChange} />
        </div>
    );
}

interface ITopicProps {
    topic: TopicEntity
    cards: CardEntity[]
    onTopicChange(): void
    onCardChange(): void
}

export default Topic;