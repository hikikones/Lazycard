import { shell } from 'electron';
import * as React from 'react';

import db from '../model/Database';
import cfg from '../model/Config';
import dialog from '../controller/Dialog';

import Button from './Button';

export default class Settings extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
        this.state = {
            dbPath: cfg.getDatabasePath()
        }
    }

    private changeDatabasePath = (): void => {
        const newPath = dialog.saveFile('lazycard', ['lazycard']);
        if (newPath === undefined) {
            return;
        }

        cfg.setDatabasePath(newPath);
        this.setState({ dbPath: newPath });
    }

    private openDatabaseDir = (): void => {
        shell.openItem(cfg.getDatabaseDir());
    }

    private changeBackupAmount = (e: React.FormEvent<HTMLInputElement>) => {
        const target = e.target as HTMLInputElement;
        let amount = Number(target.value);
        if (amount > 100) amount = 100;
        if (amount < 1) amount = 1;
        target.value = amount.toString();
        cfg.setBackupAmount(amount);
    }

    private openBackupDir = (): void => {
        shell.openItem(cfg.getBackupPath());
    }

    private restoreDatabase = () => {
        const dbFile = dialog.openFile('lazycard', ['lazycard']);
        if (dbFile === undefined) {
            return;
        }

        db.restore(dbFile);
        this.props.onTopicChange();
    }

    public render() {
        return (
            <div>
                <h1>Settings</h1>

                <h2>Database</h2>

                <h3>Path</h3>
                <p>The location of your database file.</p>
                <label>{this.state.dbPath}</label>
                <Button name="Open" icon="folder_open" action={this.openDatabaseDir} />
                <Button name="Change" icon="edit" action={this.changeDatabasePath} />

                <h3>Backups</h3>
                <p>The amount of backups before new ones starts replacing old ones.</p>
                <Button name="Open" icon="folder_open" action={this.openBackupDir} />
                <input type="number" defaultValue={cfg.getBackupAmount()} min="1" max="100" onInput={(e: React.FormEvent<HTMLInputElement>) => this.changeBackupAmount(e)} />

                <h3>Restore</h3>
                <p>Restore your database from a local file.</p>
                <Button name="Restore" icon="settings_backup_restore" action={this.restoreDatabase} />
            </div>
        );
    }
}

interface IProps {
    onTopicChange(): void
}

interface IState {
    dbPath: string
}