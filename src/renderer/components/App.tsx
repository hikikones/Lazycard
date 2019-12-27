import * as React from 'react';

import db, { Topic } from '../model/Database';
import cfg from '../model/Config';

import Nav from './Nav';
import Main from './Main';

export default class App extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);

        this.state = {
            topics: db.topics.getAll(),
            theme: `./themes/${cfg.getTheme()}.css`
        }
    }

    private updateTopics = (): void => {
        this.setState( { topics: db.topics.getAll() })
    }

    private updateTheme = (): void => {
        this.setState({ theme: `./themes/${cfg.getTheme()}.css` });
    }

    public render() {
        return (
            <div>
                <link rel="stylesheet" href={this.state.theme} />
                <Nav topics={this.state.topics} onTopicChange={this.updateTopics} />
                <Main onTopicChange={this.updateTopics} onThemeChange={this.updateTheme} />
            </div>
        );
    }
}

interface IProps {
}

interface IState {
    topics: readonly Topic[]
    theme: string
}