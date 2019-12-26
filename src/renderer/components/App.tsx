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
            theme: this.getCSSFile(cfg.getTheme())
            // TODO: also load appropriate prismjs css-file based on chosen theme
        }
        document.documentElement.setAttribute("data-theme", cfg.getTheme());

    }

    private updateTopics = (): void => {
        this.setState( { topics: db.topics.getAll() })
    }

    private updateTheme = (): void => {
        const theme = cfg.getTheme();
        const cssFile = this.getCSSFile(theme);
        document.documentElement.setAttribute("data-theme", theme);
        this.setState({ theme: cssFile });
    }

    private getCSSFile = (theme: string): string => {
        return theme === "system" ? "./system.css" : "./themes.css";
    }

    public render() {
        return (
            <div>
                <link rel="stylesheet" href={this.state.theme} type="text/css"></link>
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