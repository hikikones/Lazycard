import * as React from 'react';
import { Switch, Route } from "react-router-dom";

import db from '../model/Database';
import { Card } from '../model/Database';

import Review from './Review';
import Cards from './Cards';
import Settings from './Settings';
import Topic from './Topic';

export default class Main extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
        this.state = {
            cards: db.cards.getAll()
        }
    }

    private updateCards = () => {
        this.setState({ cards: db.cards.getAll() })
    }

    public render() {
        return (
            <main className="row row-center">
                <Switch>
                    <Route exact path={["/", "/review/:topicId"]} render={(props) => (
                        <Review {...props} key={props.match.params.topicId} />
                    )} />
                    <Route path="/cards/">
                        <Cards cards={this.state.cards} onCardChange={this.updateCards} />
                    </Route>
                    <Route path="/settings">
                        <Settings onTopicChange={this.props.onTopicChange} />
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

interface IState {
    cards: readonly Card[]
}