import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import Card from './Card';
import Button from './Button';

export default class CardEditor extends React.Component<Props, IState> {
    private readonly front = React.createRef<HTMLTextAreaElement>();
    private readonly back = React.createRef<HTMLTextAreaElement>();

    public constructor(props: Props) {
        super(props);

        const propsType = this.props;

        if (this.isCardNew(propsType)) {
            this.state = {
                front: "",
                back: ""
            }
        } else {
            this.state = {
                front: propsType.card.front,
                back: propsType.card.back
            }
        }
    }

    private isCardNew = (prop: ICardNew | ICardEdit): prop is ICardNew => {
        return (prop as ICardNew).topicId !== undefined;
    }

    private save = () => {
        if (this.isEmpty()) return;

        let card: CardEntity;
        const propsType = this.props;
        
        if (this.isCardNew(propsType)) {
            card = db.cards.new(propsType.topicId);
        } else {
            card = propsType.card;
        }

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
                    if (front) this.onInputFront();
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
            <section className="row">
                <div className="card-editor col">
                    <label>Front</label>
                    <textarea
                        ref={this.front}
                        defaultValue={this.state.front}
                        rows={4}
                        onInput={this.onInputFront}
                        onPaste={(e: React.ClipboardEvent<HTMLTextAreaElement>) => this.onPaste(e, true)}
                    />

                    <label>Back</label>
                    <textarea
                        ref={this.back}
                        defaultValue={this.state.back}
                        rows={4}
                        onInput={this.onInputBack}
                        onPaste={(e: React.ClipboardEvent<HTMLTextAreaElement>) => this.onPaste(e, false)}
                    />

                    <div className="row space-between">
                        <Button name="Save" icon="done" action={this.save} />
                        <Button name="Cancel" icon="close" action={this.cancel} />
                    </div>
                </div>
                
                <div className="col">
                    <label>Preview</label>
                    <Card front={this.state.front} back={this.state.back} />
                </div>
            </section>
        );
    }
}

interface IProps {
    onSave(): void
    onCancel(): void
}

interface ICardNew extends IProps {
    topicId: number
}

interface ICardEdit extends IProps {
    card: CardEntity
}

type Props = ICardNew | ICardEdit;

interface IState {
    front: string
    back: string
}