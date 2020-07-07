import * as React from 'react';
import { NavLink } from 'react-router-dom';

const ButtonNavLink = (props: IButtonLinkProps) => {
    const icon = () => {
        if (props.icon === undefined) return null;
        return <i className="material-icons icon">{props.icon}</i>;
    }

    const name = () => {
        if (props.name === undefined) return null;
        if (props.icon === undefined) return props.name;
        return ` ${props.name}`;
    }

    const className = (): string => {
        return props.className !== undefined ? props.className : "button";
    }

    return (
        <NavLink
            to={props.to}
            className={className()}
            isActive={props.routeName !== undefined ? (_, location) => location.pathname.includes(props.routeName) : null}
        >
            {icon()}
            {name()}
        </NavLink>
    );
}

interface IButtonLinkProps {
    to: string
    name?: string
    icon?: string
    className?: string
    routeName?: string
}

export default ButtonNavLink;