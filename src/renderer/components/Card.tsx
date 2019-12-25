import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';
import md from '../controller/Markdown';

import Dropdown, { DropdownItem } from './Dropdown';
import Modal from './Modal';
import CardEditor from './CardEditor';

export default class Card extends React.Component<ICardNew | ICard, ICardState> {
    public constructor(props: ICardNew | ICard) {
        super(props);
        this.state = {
            showModal: false
        }
    }

    private openEditor = (): void => {
        this.setState({ showModal: true });
    }

    private closeEditor = (): void => {
        this.setState({ showModal: false });
    }

    private onSave = (): void => {
        this.closeEditor();
        this.forceUpdate();
    }

    private delete = (): void => {
        if (!isCardNew(this.props)) {
            db.cards.delete(this.props.card.id);
            this.props.onDelete();
        }
    }

    public render() {
        if (isCardNew(this.props)) {
            return (
                <div className="card shadow">
                    {<CardContent markdown={this.props.front} />}
                    {<hr />}
                    {<CardContent markdown={this.props.back} />}
                </div>
            );
        }

        return (
            <div className="card shadow">
                
                <Dropdown name="" icon="more_horiz" className="card-btn" showDownArrow={false}>
                    <DropdownItem name="Edit" icon="edit" action={this.openEditor} />
                    <DropdownItem name="Delete" icon="delete" action={this.delete} />
                </Dropdown>

                {<CardContent markdown={this.props.card.front} />}
                {this.props.showBack ? <hr /> : null}
                {this.props.showBack ? <CardContent markdown={this.props.card.back} /> : null}

                <Modal show={this.state.showModal} onClickOutside={this.closeEditor}>
                    <CardEditor
                        onSave={this.onSave}
                        onCancel={this.closeEditor}
                        card={this.props.card}
                    />
                </Modal>
            </div>
        );
    }
}

interface ICardNew {
    front: string
    back: string
}

interface ICard {
    card: CardEntity
    showBack: boolean
    onDelete(): void
}

interface ICardState {
    showModal: boolean
}

const isCardNew = (prop: ICardNew | ICard): prop is ICardNew => {
    return (prop as ICardNew).front !== undefined;
}

const CardContent = (props: { markdown: string }): JSX.Element => {
    return <div dangerouslySetInnerHTML={{ __html: md.parse(props.markdown) }} />
}