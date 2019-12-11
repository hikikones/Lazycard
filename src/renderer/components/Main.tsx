import * as React from 'react';
import { Switch, Route } from "react-router-dom";

import Home from './Home';
import Cards from './Cards';
import Topic from './Topic';

export default class Main extends React.Component<IProps> {
    public constructor(props: IProps) {
        super(props);
    }

    public render() {
        return (
            <main>
                <Switch>
                    <Route exact path="/">
                        <Home />
                    </Route>
                    <Route path="/cards/">
                        <Cards />
                    </Route>
                    <Route path="/topics/:id" render={(props) => (
                        <Topic {...props} onTopicChange={this.props.onTopicChange} key={props.match.params.id} />
                    )} />
                </Switch>
            </main>
        );
    }
}

interface IProps {
    onTopicChange(): void
}