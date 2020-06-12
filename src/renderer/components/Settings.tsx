import { shell } from 'electron';
import * as React from 'react';

import Nav from './Nav';

import db from '../model/Database';
import cfg from '../model/Config';
import dialog from '../controller/Dialog';

const Settings = () => {

    const [dbPath, setDbPath] = React.useState<string>(cfg.getDatabasePath());
    
    const openDatabaseDir = () => {
        shell.openPath(cfg.getDatabaseDir());
    }

    const changeDatabasePath = () => {
        const newPath = dialog.saveFile('lazycard', ['lazycard']);
        if (newPath === undefined) {
            return;
        }

        cfg.setDatabasePath(newPath);
        setDbPath(newPath);
    }

    const openBackupDir = () => {
        shell.openPath(cfg.getBackupDir());
    }

    const changeBackupAmount = (e: React.FormEvent<HTMLInputElement>) => {
        const target = e.target as HTMLInputElement;
        let amount = Number(target.value);
        if (amount > 100) amount = 100;
        if (amount < 1) amount = 1;
        target.value = amount.toString();
        cfg.setBackupAmount(amount);
    }

    const restoreDatabase = () => {
        const dbFile = dialog.openFile('lazycard', ['lazycard']);
        if (dbFile === undefined) {
            return;
        }

        db.restore(dbFile);
    }

    return (
        <div>
            <Nav id={0} />

            <h1>Settings</h1>

            <h2>Database</h2>

            <h3>Location</h3>
            <p>The location of your database file.</p>
            <label>{dbPath}</label>
            <button onClick={openDatabaseDir}>Open</button>
            <button onClick={changeDatabasePath}>Change</button>

            <h3>Backups</h3>
            <p>The amount of backups before new ones starts replacing old ones.</p>
            <button onClick={openBackupDir}>Open</button>
            <input type="number" defaultValue={cfg.getBackupAmount()} min="1" max="100" onInput={(e: React.FormEvent<HTMLInputElement>) => changeBackupAmount(e)} />

            <h3>Restore</h3>
            <p>Restore your database from a local file.</p>
            <button onClick={restoreDatabase}>Restore</button>
        </div>
    );
}

export default Settings;