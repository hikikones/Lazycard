import React from "react";

import Config from "./../controller/Config";

export default class Main extends React.Component {
  constructor(props) {
    super(props);

    this.save = this.save.bind(this);
    this.cancel = this.cancel.bind(this);
    this.goBack = this.goBack.bind(this);

    this.backup = React.createRef();
  }

  save() {
    Config.set("backup", this.backup.current.value);
    this.goBack();
  }

  cancel() {
    this.goBack();
  }

  goBack() {
    this.props.history.goBack();
  }

  render() {
    return (
      <div>
        <h1>Settings</h1>
        <label htmlFor="backup">Backup</label>
        <input
          type="text"
          id="backup"
          ref={this.backup}
          defaultValue={Config.get("backup")}
        />

        <input
          onClick={this.save}
          type="submit"
          className="button-primary float-left"
          value="Save"
        />

        <input
          onClick={this.cancel}
          type="submit"
          className="button-primary float-right"
          value="Cancel"
        />
      </div>
    );
  }
}
