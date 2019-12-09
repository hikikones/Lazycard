import * as React from 'react';
import * as ReactDOM from 'react-dom';
import { MemoryRouter } from 'react-router-dom';

import Nav from './components/Nav';
import Main from './components/Main';
 
ReactDOM.render(
    <MemoryRouter>
        <Nav />
        <Main />
    </MemoryRouter>,
    document.getElementById('app')
);