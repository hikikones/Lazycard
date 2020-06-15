import { shell } from 'electron';
import * as React from 'react';

import db from '../model/Database';
import cfg from '../model/Config';
import dialog from '../controller/Dialog';

import Button from './Button';

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
        <div className="content">
            <h1>Settings</h1>

            <h2>Database</h2>

            <h3>Location</h3>
            <p>The location of your database file.</p>
            <p><label>{dbPath}</label></p>
            <div className="row-of-items">
                <Button name="Open" icon="folder_open" action={openDatabaseDir} />
                <Button name="Change" icon="edit" action={changeDatabasePath} />
            </div>

            <h3>Backups</h3>
            <p>The amount of backups before old ones are removed.</p>
            <div className="row-of-items">
                <Button name="Open" icon="folder_open" action={openBackupDir} />
                <input
                    type="number"
                    defaultValue={cfg.getBackupAmount()}
                    min="1"
                    max="100"
                    onInput={(e: React.FormEvent<HTMLInputElement>) => changeBackupAmount(e)}
                />
            </div>

            <h3>Restore</h3>
            <p>Restore your database from a local file.</p>
            <Button name="Restore" icon="settings_backup_restore" action={restoreDatabase} />
        </div>
    );
}

export default Settings;