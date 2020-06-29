import * as React from 'react';
import { Switch, Route, Redirect } from "react-router-dom";

import db from '../model/Database';
import { Card } from '../model/Database';

import Review from './Review';
import Cards from './Cards';
import Settings from './Settings';
import Topics from './Topics';

const Routes = () => {
    return (
        <Switch>
            <Route exact path={"/"}>
                <Redirect to="/review/0" />
            </Route>
            <Route path={"/review/:topicId"}>
                <Review />
            </Route>
            <Route exact path="/cards">
                <CardsContainer />
            </Route>
            <Route path="/settings">
                <Settings />
            </Route>
            <Route path="/topics">
                <Topics />
            </Route>
        </Switch>
    );
}

const CardsContainer = () => {
    const [cards, setCards] = React.useState<Card[]>([...db.cards.getAll()]);

    const updateCards = () => {
        setCards([...db.cards.getAll()]);
    }

    return (
        <div className="content">
            <Cards cards={cards} onCardChange={updateCards} />
        </div>
    );
}

export default Routes;