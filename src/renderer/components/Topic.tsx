import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity, Topic as TopicEntity } from '../model/Database';

import EditableHeader from './EditableHeader';
import Cards from './Cards';
import CardEditor from './CardEditor';
import Button from './Button';
import ButtonLink from './ButtonLink';
import Dropdown, { DropdownItem } from './Dropdown';

// TODO: Display something nice when deleted

const Topic = (props: ITopicProps) => {
    const [showCardEditor, setShowCardEditor] = React.useState<boolean>(false);
    const [isDeleted, setIsDeleted] = React.useState<boolean>(false);

    React.useEffect(() => {
        setIsDeleted(false);
    }, [props.topic]);

    const toggleCardEditor = () => {
        setShowCardEditor(show => !show);
    }

    const onNameChange = (newName: string) => {
        if (newName === "" || newName === props.topic.name) return;
        props.topic.name = newName;
        props.onTopicChange();
    }

    const onDelete = () => {
        db.topics.delete(props.topic.id);
        props.cards.forEach(c => db.cards.delete(c.id));
        props.onTopicChange();
        setIsDeleted(true);
    }

    if (isDeleted) {
        return (
            <div className="content">
                <h2>Topic has been deleted.</h2>
            </div>
        );
    }

    return (
        <div className="col col-center content">
            <EditableHeader title={props.topic.name} onSubmit={onNameChange} />

            <section className="row row-center col-center wrap space-fixed">
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