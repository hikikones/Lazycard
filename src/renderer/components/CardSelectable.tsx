import * as React from 'react';

import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';

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