import * as React from 'react';
import { NavLink as ReactRouterNavLink } from "react-router-dom";

const Nav = () => {
    return (
        <nav>
            <NavLink to="/review/0" routeName="review" icon="drafts" />
            <NavLink to="/cards" icon="layers" />
            <NavLink to="/topics" icon="dashboard" />
            <NavLink to="/settings" icon="settings" />
        </nav>
    );
}

const NavLink = (props: {to: string, icon: string, routeName?: string}) => {
    if (props.routeName !== undefined) {
        return (
            <ReactRouterNavLink to={props.to} className="nav" isActive={(match, location) => location.pathname.includes(props.routeName)}>
                <i className="material-icons icon">{props.icon}</i>
            </ReactRouterNavLink>
        );
    }

    return (
        <ReactRouterNavLink to={props.to} className="nav">
            <i className="material-icons icon">{props.icon}</i>
        </ReactRouterNavLink>
    );
}

export default Nav;