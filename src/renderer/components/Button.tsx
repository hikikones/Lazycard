import * as React from 'react';

const Button = (props: IButtonProps) => {
    return (
        <a className="button" onClick={props.action}>
                <i
                    className="material-icons icon"
                    style={props.color === undefined ? null : { color: props.color }}
                >
                    {props.icon}
                </i>
                {props.name === undefined ? null : ` ${props.name}`}
        </a>
    );
}

interface IButtonProps {
    icon: string
    action(): void
    name?: string
    color?: string
}

export default Button;