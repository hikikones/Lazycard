import React from 'react';
import Navigation from './Navigation';
import Main from './Main';

export default class App extends React.Component {
  render() {
    return (
      <div>
        <Navigation />
        <Main />
      </div>
    );
  }
}
