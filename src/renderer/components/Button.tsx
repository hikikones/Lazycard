import * as React from 'react';

const Button = (props: IButtonProps) => {
    const click = () => {
        props.action();
    }

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.keyCode === props.shortcut)
            click();
    }

    if (props.shortcut !== undefined) {
        React.useEffect(() => {
            window.addEventListener("keydown", onKeyDown);
            return () => {
                window.removeEventListener("keydown", onKeyDown);
            }
        }, [props.shortcut]);
    }

    return (
        <a
            className={props.className === undefined ? "button" : props.className}
            onClick={click}
            href="#"
        >
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
    shortcut?: number
    className?: string
}

export default Button;