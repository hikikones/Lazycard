import * as React from 'react';
import { Switch, Route } from "react-router-dom";

import Home from './Home';
import Topic from './Topic';

export default class Main extends React.Component {
    public render() {
        return (
            <main>
                <Switch>
                    <Route exact path="/">
                        <Home />
                    </Route>
                    <Route path="/topics/:id" render={(props) => (
                        <Topic key={props.match.params.id} {...props} />
                    )} />
                </Switch>
            </main>
        );
    }
}