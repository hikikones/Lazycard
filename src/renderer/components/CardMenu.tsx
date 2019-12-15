import * as React from 'react';

import { Card } from '../model/Database';

export default class CardMenu extends React.Component<IProps> {
    public constructor(props: IProps) {
        super(props);
    }

    public render() {
        return (
            <div>
                <button onClick={() => this.props.onEdit(this.props.card)}>Edit</button>
                <button onClick={() => this.props.onDelete(this.props.card)}>Delete</button>
            </div>
        );
    }
}

interface IProps {
    card: Card
    onEdit(card: Card): void
    onDelete(card: Card): void
}