import * as React from 'react';
import { Switch, Route } from "react-router-dom";

import Review from './Review';
import Draw from './Draw';
import Settings from './Settings';

const App = () => {
    return (
        <main>
            <Switch>
                <Route exact path="/">
                    <Review />
                </Route>
                <Route path="/draw/:id" render={(props) => (
                    <Draw {...props} key={props.match.params.id} />
                )} />
                <Route path="/settings">
                    <Settings />
                </Route>
            </Switch>
        </main>
    );
}

export default App;