import * as React from 'react';
import { NavLink as ReactRouterNavLink } from "react-router-dom";

// TODO: Fix NavLink to '/review/0' so that all topicIds are matched, not only just 0

const Nav = () => {
    return (
        <nav>
            <NavLink to="/review/0" icon="drafts" />
            <NavLink to="/cards" icon="layers" />
            <NavLink to="/topics" icon="dashboard" />
            <NavLink to="/settings" icon="settings" />
        </nav>
    );
}

const NavLink = (props: {to: string, icon: string}) => {
    return (
        <ReactRouterNavLink exact to={props.to} className="nav">
            <i className="material-icons icon">{props.icon}</i>
        </ReactRouterNavLink>
    );
}

const NavButton = (props: {action(): void, icon: string}) => {
    return (
        <a className="nav" onClick={props.action}>
            <i className="material-icons icon">{props.icon}</i>
        </a>
    );
}

export default Nav;