import * as React from 'react';
import { NavLink as ReactRouterNavLink } from "react-router-dom";

const Nav = () => {
    return (
        <nav className="col space-between">
            <div>
                <NavLink to="/review/0" icon="drafts" routeName="review" />
                <NavLink to="/cards" icon="layers" />
                <NavLink to="/topics" icon="dashboard" />
            </div>
            <div>
                <NavLink to="/settings" icon="settings" />
            </div>
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