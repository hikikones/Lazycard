import * as React from 'react';

// TODO: Fix styling

const Dropdown = (props: IDropdownProps) => {
    const [showMenu, setShowMenu] = React.useState<boolean>(false);

    const open = () => {
        setShowMenu(true);
    }

    const close = () => {
        setShowMenu(false);
    }

    return (
        <div className={props.className !== undefined ? `dropdown ${props.className}` : "dropdown"}>
            <a className="button" onClick={open}>
                <i
                    className="material-icons icon"
                    style={props.color === undefined ? null : { color: props.color }}
                >
                    {props.icon}
                </i>
                {` ${props.name}`}
                {props.showDownArrow
                    ?   <i
                            className="material-icons icon"
                            style={props.color === undefined ? null : { color: props.color }}
                        >
                            keyboard_arrow_down
                        </i>
                    :   null
                }
            </a>
            {showMenu
                ?   <DropdownMenu onNextClick={close}>
                        {props.children}
                    </DropdownMenu>
                :   null
            }
        </div>
    );
}

const DropdownMenu = (props: IDropdownMenuProps) => {
    React.useEffect(() => {
        window.addEventListener("click", props.onNextClick)
        return () => {
            window.removeEventListener("click", props.onNextClick)
        }
    });

    return (
        <div className="dropdown-menu col">
            {props.children}
        </div>
    );
}

export const DropdownItem = (props: IDropdownItemProps) => {
    return (
        <a className="button" onClick={props.action}>
            <i className="material-icons icon">{props.icon}</i> {props.name}
        </a>
    );
}

interface IDropdownProps {
    name: string
    icon: string
    showDownArrow: boolean
    children: React.ReactNode
    className?: string
    color?: string
}

interface IDropdownMenuProps {
    onNextClick(): void
    children: React.ReactNode
}

interface IDropdownItemProps {
    name: string
    icon: string
    action(): void
}

export default Dropdown;