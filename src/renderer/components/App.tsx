import * as React from 'react';

import db from '../model/Database';
import { Topic } from '../model/Database';

import Nav from './Nav';
import Main from './Main';

export default class App extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);

        this.state = {
            topics: db.topics.getAll()
        }
    }

    private updateTopics = () => {
        this.setState( { topics: db.topics.getAll() })
    }

    public render() {
        return (
            <div>
                <Nav topics={this.state.topics} onTopicChange={this.updateTopics} />
                <Main onTopicChange={this.updateTopics} />
            </div>
        );
    }
}

interface IProps {
}

interface IState {
    topics: readonly Topic[]
}