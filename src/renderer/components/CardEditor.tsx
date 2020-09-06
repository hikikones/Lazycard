import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';

import CardView from './CardView';
import Button from './Button';

const CardEditor = (props: ICardEditorProps) => {
    const [front, setFront] = React.useState<string>(isNewCard(props) ? "" : props.card.front);

    const frontInput = React.useRef<HTMLTextAreaElement>();

    const onFrontChange = () => {
        setFront(frontInput.current.value);
    }

    const save = () => {
        if (isEmpty()) return;

        const card = isNewCard(props) ? db.cards.new(props.topicId) : props.card;
        card.front = frontInput.current.value;
        
        clear();
        frontInput.current.focus();
        props.onSave();
    }

    const cancel = () => {
        props.onCancel();
    }

    const clear = () => {
        frontInput.current.value = "";
        onFrontChange();
    }

    const isEmpty = (): boolean => {
        return frontInput.current.value === "";
    }

    const onPaste = (e: React.ClipboardEvent<HTMLTextAreaElement>, front: boolean) => {
        for (let i = 0 ; i < e.clipboardData.items.length; i++) {
            const item = e.clipboardData.items[i];

            if (item.type.indexOf("image") === -1)
                continue;

            e.preventDefault();
            const reader = new FileReader();
            reader.readAsDataURL(item.getAsFile());
            e.persist();

            reader.onloadend = () => {
                const base64 = reader.result;
                const target = e.target as HTMLTextAreaElement;
                target.value += `![](${base64})`;
                if (front) onFrontChange();
            }
        }
    }

    return (
        <section className="row space-fixed">
            <section className="col">
                <label>Front</label>
                <textarea
                    ref={frontInput}
                    className="card-textarea"
                    defaultValue={front}
                    rows={5}
                    onChange={onFrontChange}
                    onPaste={(e: React.ClipboardEvent<HTMLTextAreaElement>) => onPaste(e, true)}
                />
                <section className="row space-between">
                    <Button name="Save" icon="done" action={save} />
                    <Button name="Done" icon="close" action={cancel} />
                </section>
            </section>

        </section>
    );
}

interface IPropsCommon {
    onSave(): void
    onCancel(): void
}

interface ICardNew extends IPropsCommon {
    topicId: number
}

interface ICardEdit extends IPropsCommon {
    card: CardEntity
}

type ICardEditorProps = ICardNew | ICardEdit;

const isNewCard = (prop: ICardEditorProps): prop is ICardNew => {
    return (prop as ICardNew).topicId !== undefined;
}

export default CardEditor;