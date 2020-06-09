import { ipcRenderer } from 'electron';
import * as React from 'react';
import * as ReactDOM from 'react-dom';

import App from './components/App';
import "./style.css";
 
ReactDOM.render(
    <App />,
    document.getElementById('app')
);

ipcRenderer.on('app-close', () => {
    ipcRenderer.send('quit');
});