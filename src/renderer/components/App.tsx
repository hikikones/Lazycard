import * as React from 'react';
import { NavLink, Switch, Route } from "react-router-dom";

import Review from './Review';
import Draw from './Draw';
import Settings from './Settings';

const App = () => {
    return (
        <div>
            <nav>
                <NavLink to="/">
                    <button>Review</button>
                </NavLink>
                <NavLink to="/draw/0">
                    <button>Add</button>
                </NavLink>
                <NavLink to="/settings">
                    <button>Settings</button>
                </NavLink>
            </nav>
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
        </div>
    );
}

export default App;