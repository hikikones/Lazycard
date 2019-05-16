import React from "react";
import { NavLink } from "react-router-dom";

export default class Navigation extends React.Component {
  render() {
    return (
      <nav>
        <ul>
          <li>
            <NavLink exact to="/">
              <i className="material-icons">home</i>
            </NavLink>
          </li>
          <li>
            <NavLink to="/topic/0">
              <i className="material-icons">view_quilt</i>
            </NavLink>
          </li>
          <li>
            <NavLink to="/topic/submit/0">
              <i className="material-icons">playlist_add</i>
            </NavLink>
          </li>
          <li>
            <NavLink to="/card/submit/0/0">
              <i className="material-icons">add_circle</i>
            </NavLink>
          </li>
          <li>
            <NavLink to="/settings">
              <i className="material-icons">settings</i>
            </NavLink>
          </li>
        </ul>
      </nav>
    );
  }
}
