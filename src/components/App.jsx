import React from 'react';
import Header from './Header';
import Navigation from './Navigation';
import Main from './Main';

export default class App extends React.Component {
  render() {
    return (
      <div>
        <Header />
        <Navigation />
        <Main />
      </div>
    );
  }
}
