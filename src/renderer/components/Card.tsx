import * as React from 'react';

import db from '../model/Database';
import { Card as CardEntity } from '../model/Database';
import md from '../controller/Markdown';

import Button from './Button';
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
                <CardMenu onEdit={this.openEditor} onDelete={this.delete} />
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

const CardMenu = (props: { onEdit(): void, onDelete(): void }): JSX.Element => {
    const [showMenu, toggleMenu] = React.useState(false);

    const onEdit = () => {
        toggleMenu(false);
        props.onEdit();
    }

    const onDelete = () => {
        toggleMenu(false);
        props.onDelete();
    }

    return (
        <div className="card-btn">
            <Button name="" icon="more_horiz" action={() => toggleMenu(!showMenu)} />
            {showMenu
                ?   <div className="card-menu shadow">
                        <a href="#" onClick={onEdit} className="nav">
                            <i className="material-icons">edit</i> Edit
                        </a>
                        <a href="#" onClick={onDelete} className="nav">
                            <i className="material-icons">delete</i> Delete
                        </a>
                    </div>
                :   null
            }
        </div>
    );
}