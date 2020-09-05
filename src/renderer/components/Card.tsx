import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import CardView from './CardView';
import Dropdown, { DropdownItem } from './Dropdown';
import Modal from './Modal';
import CardEditor from './CardEditor';
import Button from './Button';

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
            <CardView front={props.card.front}>
                 <Button name="Audio" icon="volume_up" action={null} />
                    <Button name="Edit" icon="edit" action={toggleEditor} />
                    <Button name="Delete" icon="delete" action={onDelete} />

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