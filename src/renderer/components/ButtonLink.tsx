import * as React from 'react';
import { Link } from 'react-router-dom';

const ButtonLink = (props: IButtonLinkProps) => {
    const icon = () => {
        if (props.icon === undefined) return null;
        return <i className="material-icons icon">{props.icon}</i>;
    }

    const name = () => {
        if (props.icon === undefined) return props.name;
        return ` ${props.name}`;
    }

    return (
        <Link to={props.to} className={props.className !== undefined ? props.className : "button"} >
            {icon()}
            {name()}
        </Link>
    );
}

interface IButtonLinkProps {
    name: string
    to: string
    icon?: string
    className?: string
}

export default ButtonLink;