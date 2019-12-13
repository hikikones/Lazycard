import { remote } from "electron";

class Dialog {
    public saveFile(name: string, extensions: string[]): string {
        return remote.dialog.showSaveDialogSync(remote.getCurrentWindow(), {
            filters: [{ name: name, extensions: extensions }]
        })
    }
}

export default new Dialog();