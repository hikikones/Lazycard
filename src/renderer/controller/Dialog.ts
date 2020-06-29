import { remote } from "electron";

// TODO: default filename

class Dialog {
    public saveFile(name: string, extensions: string[]): string {
        return remote.dialog.showSaveDialogSync(remote.getCurrentWindow(), {
            filters: [{ name: name, extensions: extensions }]
        })
    }

    public openFile(name: string, extensions: string[]): string {
        const paths = remote.dialog.showOpenDialogSync(remote.getCurrentWindow(), {
            filters: [{ name: name, extensions: extensions }],
            properties: ['openFile']
        });
        
        if (paths === undefined) {
            return undefined;
        }

        return paths[0];
    }

    public openDirectory(): string {
        const paths = remote.dialog.showOpenDialogSync(remote.getCurrentWindow(), {
            properties: ['openDirectory', 'createDirectory']
        });
        
        if (paths === undefined) {
            return undefined;
        }

        return paths[0];
    }
}

export default new Dialog();