import * as React from 'react';
import { Switch, Route, Redirect } from "react-router-dom";

import db from '../model/Database';
import { Card } from '../model/Database';

import Review from './Review';
import Cards from './Cards';
import Settings from './Settings';
import Topics from './Topics';
import Input from './Input';

const Main = (props: IMainProps) => {
    return (
        <Switch>
            <Route exact path={"/"}>
                <Redirect to="/review/0" />
            </Route>
            <Route path={"/review/:topicId"}>
                <Review />
            </Route>
            <Route path="/cards">
                <CardsContainer />
            </Route>
            <Route path="/settings">
                <Settings onThemeChange={props.onThemeChange} />
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
        <main>
            {cards.length > 0 && <h1 className="text-center">Cards</h1>}
            <Cards cards={cards} onCardChange={updateCards} />
        </main>
    );
}

interface IMainProps {
    onThemeChange(): void
}

export default Main;