import * as React from 'react';
import { Link } from 'react-router-dom';

const ButtonLink = (props: IButtonLinkProps) => {
    return (
        <Link className="button" to={props.to}>
            <i className="material-icons icon">{props.icon}</i> {props.name}
        </Link>
    );
}

interface IButtonLinkProps {
    icon: string
    name: string
    to: string
}

export default ButtonLink;