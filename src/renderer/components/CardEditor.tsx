import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';
import Card from './Card';

export default class CardEditor extends React.Component<IProps, IState> {
    private readonly front = React.createRef<HTMLTextAreaElement>();
    private readonly back = React.createRef<HTMLTextAreaElement>();

    public constructor(props: IProps) {
        super(props);
        this.state = {
            front: "",
            back: ""
        }
    }

    private save = () => {
        if (this.isEmpty()) return;

        const card: CardEntity = db.cards.new(this.props.topicId);
        card.front = this.front.current.value;
        card.back = this.back.current.value;
        this.clear();
        this.props.onSave();
    }

    private cancel = () => {
        this.clear();
        this.props.onCancel();
    }

    private handleOnInput = () => {
        this.setState({
            front: this.front.current.value,
            back: this.back.current.value
        });
    }

    private clear = () => {
        this.front.current.value = "";
        this.back.current.value = "";
        this.handleOnInput();
    }

    private isEmpty = (): boolean => {
        return this.front.current.value === "" || this.back.current.value === "";
    }

    public render() {
        return (
            <div>
                <label>Front</label>
                <textarea ref={this.front} onInput={this.handleOnInput} />

                <label>Back</label>
                <textarea ref={this.back} onInput={this.handleOnInput} />

                <button onClick={this.save}>Save</button>
                <button onClick={this.cancel}>Cancel</button>

                <label>Preview</label>
                <Card front={this.state.front} back={this.state.back} />
            </div>
        );
    }
}

interface IProps {
    topicId: number
    onSave(): void
    onCancel(): void
}

interface IState {
    front: string
    back: string
}