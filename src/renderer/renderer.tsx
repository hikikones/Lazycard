import { ipcRenderer } from 'electron';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import { MemoryRouter } from 'react-router-dom';

import db from './model/Database';
import App from './components/App';
import "./style.css";
 
ReactDOM.render(
    <MemoryRouter>
        <App />
    </MemoryRouter>,
    document.getElementById('app')
);

ipcRenderer.on('app-close', () => {
    db.save();
    ipcRenderer.send('quit');
});