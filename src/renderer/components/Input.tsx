import * as React from 'react';

// TODO: fix classnames

const Input = (props: IInputProps) => {
    const input = React.useRef<HTMLInputElement>();

    const clear = () => {
        input.current.value = "";
        if (props.onClear !== undefined) props.onClear();
    }

    return (
        <div className="input-container col-center">
            {props.icon && <i className="material-icons">{props.icon}</i>}
            <input
                ref={input}
                className={props.className || null}
                placeholder={props.placeholder || null}
                type="text"
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => props.onChange(e.target.value)}
            />
            <i onClick={clear} className="material-icons clear-icon">close</i>
        </div>
    );
}

interface IInputProps {
    onChange(newValue: string): void
    className?: string
    placeholder?: string
    icon?: string
    onClear?(): void
}

export default Input;