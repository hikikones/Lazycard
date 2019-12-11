import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';
import { Card } from '../model/Database';

import Cards from './Cards';
import CardEditor from './CardEditor';

export default class Topic extends React.Component<IProps, IState> {
    private readonly topic = db.topics.get(parseInt(this.props.match.params.id));

    public constructor(props: IProps) {
        super(props);
        this.state = {
            name: this.topic.name,
            showCardEditor: false,
            cards: db.cards.getByTopic(this.topic.id)
        }
    }

    private changeName = (newName: string): void => {
        this.topic.name = newName;
        this.setState({ name: newName });
        this.props.onTopicChange();
    }

    private updateCards = () => {
        this.setState({ cards: db.cards.getByTopic(this.topic.id) });
    }

    private toggleCardEditor = () => {
        this.setState({ showCardEditor: !this.state.showCardEditor });
    }

    public render() {
        return (
            <div>
                <h1>{this.state.name}</h1>

                {this.state.showCardEditor
                    ?   <CardEditor
                            topicId={this.topic.id}
                            onSave={this.updateCards}
                            onCancel={this.toggleCardEditor}
                        />
                    :   <button onClick={this.toggleCardEditor}>Add new card</button>
                }

                <button onClick={() => this.changeName("New Name")}>Change name</button>

                <Cards cards={this.state.cards} />
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
}