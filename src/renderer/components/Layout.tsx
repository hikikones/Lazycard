import * as React from 'react';
import { NavLink } from "react-router-dom";

const Layout = (props: ILayoutProps) => {
    return (
        <div
            className="layout"
            style={{ gridTemplateColumns: `${props.sidebarWidth}px auto` }}
        >
            {props.children}
        </div>
    );
}

interface ILayoutProps {
    sidebarWidth: number
    children: React.ReactNode
}

export const Sidebar = (props: {children: React.ReactNode}) => {
    return (
        <div className="layout-sidebar">
            {props.children}
        </div>
    );
}

export const SidebarItem = (props: ISidebarItemProps) => {
    const name = (): string => {
        if (props.icon === undefined) return props.name;
        return ` ${props.name}`;
    }

    return (
        <a onClick={props.onClick} className={props.active ? "sidebar-item active" : "sidebar-item"}>
            {props.icon && <i className="material-icons icon">{props.icon}</i>}
            {name()}
        </a>
    );
}

interface ISidebarItemProps {
    name: string
    active: boolean
    onClick(): void
    icon?: string
}

export const Content = (props: {children: React.ReactNode}) => {
    return (
        <div className="layout-main">
            {props.children}
        </div>
    );
}

export const SidebarLink = (props: {to: string, name: string, icon?: string}) => {
    const name = (): string => {
        if (props.icon === undefined) return props.name;
        return ` ${props.name}`;
    }
    
    return (
        <NavLink to={props.to} className="sidebar-item">
            {props.icon && <i className="material-icons icon">{props.icon}</i>}
            {name()}
        </NavLink>
    );
}

export default Layout;