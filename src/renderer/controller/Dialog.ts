import { remote } from "electron";

class Dialog {
    public saveFile(name: string, extensions: string[]): string {
        return remote.dialog.showSaveDialogSync(remote.getCurrentWindow(), {
            filters: [{ name: name, extensions: extensions }]
        })
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