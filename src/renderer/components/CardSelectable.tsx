import * as React from 'react';

import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';
import { truncate } from 'fs';

const CardSelectable = (props: ICardSelectableProps) => {
    const [showCheckbox, setShowCheckbox] = React.useState<boolean>(true);

    const toggleSelect = () => {
        props.card.selected = !props.card.selected;
        if (props.card.selected) props.onSelect();
        else props.onDeselect();
    }

    return (
        <div onMouseEnter={() => setShowCheckbox(true)} onMouseLeave={() => setShowCheckbox(true)}>
            <Card
                card={props.card}
                onDelete={() => props.onDelete(props.card)}
            >
                {(props.card.selected || showCheckbox) &&
                    <Button
                        icon={props.card.selected ? "check_box" : "check_box_outline_blank"}
                        action={toggleSelect}
                        className="button card-checkbox"
                    />
                }
            </Card>
        </div>
    );
}

interface ICardSelectableProps {
    card: CardEntity
    onDelete(card: CardEntity): void
    onSelect(): void
    onDeselect(): void
}

export default CardSelectable;