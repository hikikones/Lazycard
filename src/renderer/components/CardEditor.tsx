import * as React from 'react';

import db from '../model/Database';
import { Card } from '../model/Database';

export default class CardEditor extends React.Component<IProps> {
    private front = React.createRef<HTMLTextAreaElement>();
    private back = React.createRef<HTMLTextAreaElement>();

    public constructor(props: IProps) {
        super(props);
    }

    private save = () => {
        if (this.isEmpty()) return;

        const card: Card = new Card(this.props.topicId);
        card.front = this.front.current.value;
        card.back = this.back.current.value;
        db.cards.add(card);
        this.clear();
        this.props.onSave();
    }

    private cancel = () => {
        this.clear();
        this.props.onCancel();
    }

    private clear = () => {
        this.front.current.value = "";
        this.back.current.value = "";
    }

    private isEmpty = (): boolean => {
        return this.front.current.value === "" || this.back.current.value === "";
    }

    public render() {
        return (
            <div>
                <label>Front</label>
                <textarea ref={this.front} />

                <label>Back</label>
                <textarea ref={this.back} />

                <button onClick={this.save}>Save</button>
                <button onClick={this.cancel}>Cancel</button>
            </div>
        );
    }
}

interface IProps {
    topicId: number
    onSave(): void
    onCancel(): void
}