import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import CardView from './CardView';
import Dropdown, { DropdownItem } from './Dropdown';
import Modal from './Modal';
import CardEditor from './CardEditor';

const Card = (props: ICardProps) => {
    const [showEditor, setShowEditor] = React.useState<boolean>(false);

    const toggleEditor = () => {
        setShowEditor(show => !show);
        if (props.onToggleModal !== undefined) props.onToggleModal();
    }

    const onDelete = () => {
        db.cards.delete(props.card.id);
        props.onDelete();
    }

    return (
        <>
            <CardView front={props.card.front} time={props.card.time}>
                <Dropdown name="" icon="more_horiz" className="card-dropdown" showDownArrow={false}>
                    <DropdownItem name="Edit" icon="edit" action={toggleEditor} />
                    <DropdownItem name="Delete" icon="delete" action={onDelete} />
                </Dropdown>
                <div hidden={!(props.card.time && props.card.time > 0)}>{`Time to complete task: ${props.card.time} mins`}</div>
                {props.children || null}
            </CardView>

            <Modal show={showEditor} onClickOutside={toggleEditor}>
                <CardEditor
                    onSave={toggleEditor}
                    onCancel={toggleEditor}
                    card={props.card}
                />
            </Modal>
        </>
    );
}

interface ICardProps {
    card: CardEntity
    onDelete(): void
    onToggleModal?(): void
    children?: React.ReactNode
}

export default Card;