import React from "react";

import Statistics from "./Statistics";

export default class Home extends React.Component {
  render() {
    return (
      <div>
        <h1 className="text-center">Home</h1>
        <Statistics topicId={0} />
      </div>
    );
  }
}
