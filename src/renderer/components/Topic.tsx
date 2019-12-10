import * as React from 'react';
import { RouteComponentProps } from 'react-router-dom';

import db from '../model/Database';

import Cards from './Cards';

export default class Topic extends React.Component<IProps, IState> {
    private readonly topic = db.topics.get(parseInt(this.props.match.params.id));

    public constructor(props: IProps) {
        super(props);
        this.state = {
            name: this.topic.name
        }
    }

    private test = (): void => {
        this.setState({ name: "LOLOLOLOLOLOL" });
    }

    public render() {
        return (
            <div>
                <h1>{this.state.name}</h1>
                <button onClick={this.test}>Ok</button>

                <Cards topicId={this.topic.id} />
            </div>
        );
    }
}

interface IProps extends RouteComponentProps<{ id: string }> {
}

interface IState {
    name: string
}