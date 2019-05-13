import React from 'react';
import { Switch, Route } from 'react-router-dom';

import Home from './Home';
import Topics from './Topics';
import TopicReview from './TopicReview';
import TopicSubmit from './TopicSubmit';
import CardSubmit from './CardSubmit';

export default class Main extends React.Component {
  render() {
    return (
      <main>
        <div className="main-content">
          <Switch>
            <Route exact path="/" component={Home} />
            <Route exact path="/topic/:topic_id" component={Topics} />
            <Route exact path="/topic/submit/:topic_id" component={TopicSubmit} />
            <Route exact path="/topic/review/:topic_id" component={TopicReview} />
            <Route exact path="/card/submit/:card_id/:topic_id" component={CardSubmit} />
          </Switch>
        </div>
      </main>
    );
  }
}
