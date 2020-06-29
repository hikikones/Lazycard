import { ipcRenderer } from 'electron';
import * as React from 'react';
import * as ReactDOM from 'react-dom';

import db from './model/Database';
import cfg from './model/Config';
import App from './components/App';
import './style.css';
 
ReactDOM.render(
        <App />,
    document.getElementById('app')
);

ipcRenderer.on('app-close', () => {
    db.save();
    cfg.save();
    ipcRenderer.send('quit');
});