import * as React from 'react';

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
    return (
        <a onClick={props.onClick} className={props.active ? "sidebar-item active" : "sidebar-item"}>
            <i className="material-icons">{props.icon}</i> {props.name}
        </a>
    );
}

interface ISidebarItemProps {
    name: string
    icon: string
    active: boolean
    onClick(): void
}

export const Content = (props: {children: React.ReactNode}) => {
    return (
        <div className="layout-main">
            {props.children}
        </div>
    );
}

export default Layout;