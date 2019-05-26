import React from "react";

import Statistics from "./Statistics";

export default class Home extends React.Component {
  render() {
    return (
      <div>
        <h2 className="text-center">Home</h2>
        <Statistics topicId={0} />
      </div>
    );
  }
}
