import * as React from 'react';

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

    public render() {
        return (
            <div>
                <h1>Settings</h1>

                <h2>Database</h2>
                <label>{this.state.dbPath}</label>
                <Button name="Change" icon="edit" action={this.changeDatabasePath} />
            </div>
        );
    }
}

interface IProps {
}

interface IState {
    dbPath: string
}