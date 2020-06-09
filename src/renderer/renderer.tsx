import { ipcRenderer } from 'electron';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
 
ReactDOM.render(
    <h1>Hello!</h1>,
    document.getElementById('app')
);

ipcRenderer.on('app-close', () => {
    ipcRenderer.send('quit');
});