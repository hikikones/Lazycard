import * as React from 'react';
import { NavLink as ReactRouterNavLink } from "react-router-dom";

const Nav = () => {
    return (
        <nav>
            <NavLinkWithId to="/review/0" icon="drafts" />
            <NavLink to="/cards" icon="layers" />
            <NavLink to="/topics" icon="dashboard" />
            <NavLink to="/settings" icon="settings" />
        </nav>
    );
}

const NavLink = (props: {to: string, icon: string}) => {
    return (
        <ReactRouterNavLink to={props.to} className="nav">
            <i className="material-icons icon">{props.icon}</i>
        </ReactRouterNavLink>
    );
}

const NavLinkWithId = (props: {to: string, icon: string}) => {
    return (
        <ReactRouterNavLink
            to={props.to}
            className="nav"
            isActive={(match, location) => {
                const tokens = location.pathname.split('/');
                const id = tokens[tokens.length - 1];
                return !isNaN(Number(id));
            }}
        >
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