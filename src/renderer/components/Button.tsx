import * as React from 'react';

const Button = (props: IButtonProps) => {
    const [toggle, setToggle] = React.useState<boolean>(props.toggle || true);

    const click = () => {
        if (props.icon2 !== undefined) setToggle(t => !t);
        props.action();
    }

    const icon = (): string => {
        if (toggle) return props.icon;
        return props.icon2;
    }

    const name = (): string => {
        if (props.name === undefined) return null;
        return ` ${props.name}`;
    }

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.keyCode === props.shortcut) click();
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
                    {icon()}
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
    icon2?: string
    toggle?: boolean
    shortcut?: number
}

export default Button;