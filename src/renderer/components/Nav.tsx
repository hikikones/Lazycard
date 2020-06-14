import * as React from 'react';
import { NavLink } from "react-router-dom";

const Nav = () => {
    return (
        <nav>
            <NavButton to="/" icon="drafts" />
            <NavButton to="/draw/0" icon="add" />
            <NavButton to="/settings" icon="settings" />
        </nav>
    );
}

const NavButton = (props: {to: string, icon: string}) => {
    return (
        <NavLink exact to={props.to}>
            <i className="material-icons">{props.icon}</i>
        </NavLink>
    );
}

export default Nav;