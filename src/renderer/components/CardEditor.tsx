import * as React from 'react';

import db from '../model/Database';
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
        const card = db.cards.new(this.props.topicId);
        card.front = this.front.current.value;
        card.back = this.back.current.value;
        this.clear();
        this.props.onSave();
    }

    private cancel = () => {
        this.clear();
        this.props.onCancel();
    }

    private onInputFront = () => {
        this.setState({ front: this.front.current.value });
    }

    private onInputBack = () => {
        this.setState({ back: this.back.current.value });
    }

    private onPaste = (e: React.ClipboardEvent<HTMLTextAreaElement>, front: boolean) => {
        for (let i = 0 ; i < e.clipboardData.items.length; i++) {
	        const item = e.clipboardData.items[i];
	        if (item.type.indexOf("image") !== -1) {
                e.preventDefault();
                const reader = new FileReader();
                reader.readAsDataURL(item.getAsFile());
                e.persist();
                reader.onloadend = () => {
                    const base64 = reader.result;
                    const target = e.target as HTMLTextAreaElement;
                    target.value += `![](${base64})`;
                    if (front)  this.onInputFront();
                    else this.onInputBack();
                }
	        }
	    }
    }

    private clear = () => {
        this.front.current.value = "";
        this.back.current.value = "";
        this.onInputFront();
        this.onInputBack();
    }

    private isEmpty = (): boolean => {
        return this.front.current.value === "" || this.back.current.value === "";
    }

    public render() {
        return (
            <div>
                <label>Front</label>
                <textarea
                    ref={this.front}
                    onInput={this.onInputFront}
                    onPaste={(e: React.ClipboardEvent<HTMLTextAreaElement>) => this.onPaste(e, true)}
                />

                <label>Back</label>
                <textarea
                    ref={this.back}
                    onInput={this.onInputBack}
                    onPaste={(e: React.ClipboardEvent<HTMLTextAreaElement>) => this.onPaste(e, false)}
                />

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
    id?: number
}

interface IState {
    front: string
    back: string
}