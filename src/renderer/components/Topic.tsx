import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card } from '../model/Database';

import Cards from './Cards';
import CardEditor from './CardEditor';
import Button from './Button';
import Dropdown, { DropdownItem } from './Dropdown';

export default class Topic extends React.Component<IProps, IState> {
    private readonly topic = db.topics.get(parseInt(this.props.match.params.id));
    private readonly nameInput = React.createRef<HTMLInputElement>();

    public constructor(props: IProps) {
        super(props);
        this.state = {
            name: this.topic.name,
            edit: false,
            showCardEditor: false,
            cards: db.cards.getByTopic(this.topic.id),
            deleted: false
        }
    }

    private enableEdit = () => {
        this.setState({ edit: true });
    }

    private submitEdit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const form = e.target as HTMLFormElement;
        const target = form.children[0] as HTMLInputElement;
        this.validateNameChange(target.value);
        this.setState({ edit: false });
    }

    private onBlurEdit = (e: React.FormEvent<HTMLInputElement>) => {
        e.preventDefault();
        const target = e.target as HTMLInputElement;
        this.validateNameChange(target.value);
        this.setState({ edit: false });
    }

    private validateNameChange = (newName: string): void => {
        if (newName === "" || db.topics.exists(newName.trim())) return;

        this.topic.name = newName;
        this.setState({ name: newName });
        this.props.onTopicChange();
    }

    private updateCards = (): void => {
        this.setState({ cards: db.cards.getByTopic(this.topic.id) });
    }

    private toggleCardEditor = (): void => {
        this.setState({ showCardEditor: !this.state.showCardEditor });
    }

    private delete = () => {
        this.setState({ deleted: true });
        db.topics.delete(this.topic.id);
        this.props.onTopicChange();
        this.state.cards.forEach(c => db.cards.delete(c.id));
    }

    public render() {
        if (this.state.deleted) {
            return (
                <div>
                    <h3>Topic has been deleted.</h3>
                </div>
            );
        }

        return (
            <div>
                {!this.state.edit
                    ?   <h1 className="topic-name" onClick={this.enableEdit}>{this.state.name}</h1>
                    :   <form onSubmit={(e: React.FormEvent<HTMLFormElement>) => this.submitEdit(e)}>
                            <input
                                ref={this.nameInput}
                                className="topic-name"
                                onBlur={(e: React.FormEvent<HTMLInputElement>) => this.onBlurEdit(e)}
                                defaultValue={this.state.name}
                                autoFocus={true}
                                type="text"
                            />
                        </form>
                }

                <section>
                    {this.state.showCardEditor ? null : <Button name="Add new card" icon="add" action={this.toggleCardEditor} />}
                    <Button name="Review" icon="drafts" to={`/review/${this.topic.id}`} />
                    <Dropdown name="Export" icon="save" showDownArrow={true}>
                        <DropdownItem name="JSON" icon="archive" action={() => db.export(this.topic.id)} />
                        <DropdownItem name="HTML" icon="file_copy" action={() => db.exportToHTML(this.topic.id)} />
                    </Dropdown>
                    <Button name="Delete" icon="delete" action={this.delete} />
                </section>

                {this.state.showCardEditor
                    ?   <div>
                            <CardEditor
                                topicId={this.topic.id}
                                onSave={this.updateCards}
                                onCancel={this.toggleCardEditor}
                            />
                        </div>
                    :   null
                }

                <Cards cards={this.state.cards} onCardChange={this.updateCards} />
            </div>
        );
    }
}

interface IProps extends RouteComponentProps<{ id: string }> {
    onTopicChange(): void
}

interface IState {
    name: string
    edit: boolean
    cards: readonly Card[]
    showCardEditor: boolean
    deleted: boolean
}