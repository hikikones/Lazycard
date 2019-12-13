import * as React from 'react';

import cfg from '../model/Config';
import dialog from '../controller/Dialog';

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

                <h2>Database Path</h2>
                <label>Current path</label>
                <p>{this.state.dbPath}</p>
                <button onClick={this.changeDatabasePath}>Change</button>
            </div>
        );
    }
}

interface IProps {
}

interface IState {
    dbPath: string
}