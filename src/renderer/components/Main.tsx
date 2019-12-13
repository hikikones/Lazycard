import * as React from 'react';
import { Switch, Route } from "react-router-dom";

import Home from './Home';
import Review from './Review';
import Cards from './Cards';
import Settings from './Settings';
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
                    <Route path="/review/:topicId" render={(props) => (
                        <Review {...props} key={props.match.params.topicId} />
                    )} />
                    <Route path="/cards/">
                        <Cards /> {/* TODO: pass cards in props? */}
                    </Route>
                    <Route path="/settings">
                        <Settings />
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