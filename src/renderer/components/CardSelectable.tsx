import * as React from 'react';

import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';

const CardSelectable = (props: ICardSelectableProps) => {
    const toggleSelect = () => {
        props.card.selected = !props.card.selected;
        if (props.card.selected) props.onSelect();
        else props.onDeselect();
    }

    return (
        <Card
            card={props.card}
            showBack={props.showBack}
            onDelete={() => props.onDelete(props.card)}
        >
            <Button
                icon={props.card.selected ? "check_box" : "check_box_outline_blank"}
                action={toggleSelect}
            />
        </Card>
    );
}

interface ICardSelectableProps {
    card: CardEntity
    showBack: boolean
    onDelete(card: CardEntity): void
    onSelect(): void
    onDeselect(): void
}

export default CardSelectable;