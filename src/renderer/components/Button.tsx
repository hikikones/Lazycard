import * as React from 'react';

const Button = (props: IButtonProps) => {
    const click = () => {
        props.action();
    }

    const name = (): string => {
        if (props.name === undefined) return null;
        return ` ${props.name}`;
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
        <a className="button" onClick={click}>
            <i
                className="material-icons icon"
                style={props.color === undefined ? null : { color: props.color }}
            >
                {props.icon}
            </i>
            {name()}
        </a>
    );
}

interface IButtonProps {
    icon: string
    action(): void
    name?: string
    color?: string
    shortcut?: number
}

export default Button;