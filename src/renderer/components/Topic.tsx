import * as React from 'react';
import { Link, RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card } from '../model/Database';

import Cards from './Cards';
import CardEditor from './CardEditor';
import Button from './Button';

export default class Topic extends React.Component<IProps, IState> {
    private readonly topic = db.topics.get(parseInt(this.props.match.params.id));

    public constructor(props: IProps) {
        super(props);
        this.state = {
            name: this.topic.name,
            showCardEditor: false,
            cards: db.cards.getByTopic(this.topic.id),
            deleted: false
        }
    }

    private changeName = (newName: string): void => {
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
                <h1>{this.state.name}</h1>

                <section>
                    {this.state.showCardEditor ? null : <Button name="Add new card" icon="add" action={this.toggleCardEditor} />}
                    <Button name="Change name" icon="edit" action={() => this.changeName("New Name")} />
                    <Button name="Export" icon="save" action={() => db.export(this.topic.id)} />
                    <Button name="Review" icon="drafts" to={`/review/${this.topic.id}`} />
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
    cards: readonly Card[]
    showCardEditor: boolean
    deleted: boolean
}