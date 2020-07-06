import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import CardView from './CardView';
import Dropdown, { DropdownItem } from './Dropdown';
import Modal from './Modal';
import CardEditor from './CardEditor';

const Card = (props: ICardProps) => {
    const [showEditor, setShowEditor] = React.useState<boolean>(false);
    const [showStats, setShowStats] = React.useState<boolean>(false);

    const toggleEditor = () => {
        setShowEditor(show => !show);
        if (props.onToggleModal !== undefined) props.onToggleModal();
    }

    const toggleStats = () => {
        setShowStats(show => !show);
        if (props.onToggleModal !== undefined) props.onToggleModal();
    }

    const onDelete = () => {
        db.cards.delete(props.card.id);
        props.onDelete();
    }

    return (
        <div>
            <CardView front={props.card.front} back={props.card.back} showBack={props.showBack}>
                <Dropdown name="" icon="more_horiz" className="card-dropdown" showDownArrow={false}>
                    <DropdownItem name="Edit" icon="edit" action={toggleEditor} />
                    <DropdownItem name="Stats" icon="assessment" action={toggleStats} />
                    <DropdownItem name="Delete" icon="delete" action={onDelete} />
                </Dropdown>

                {props.children || null}
            </CardView>

            <Modal show={showEditor} onClickOutside={toggleEditor}>
                <CardEditor
                    onSave={toggleEditor}
                    onCancel={toggleEditor}
                    card={props.card}
                />
            </Modal>

            <Modal show={showStats} onClickOutside={toggleStats}>
                <h2>Stats</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Attempts</th>
                            <th>Successes</th>
                            <th>Retention Rate</th>
                            <th>Due date</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>{props.card.attempts}</td>
                            <td>{props.card.successes}</td>
                            <td>{props.card.retentionRate().toFixed(2)}</td>
                            <td>{props.card.dueDate.toLocaleDateString()}</td>
                        </tr>
                    </tbody>
                </table>
            </Modal>
        </div>
    );
}

interface ICardProps {
    card: CardEntity
    showBack: boolean
    onDelete(): void
    onToggleModal?(): void
    children?: React.ReactNode
}

export default Card;