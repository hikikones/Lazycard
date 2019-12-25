import * as React from 'react';

export default class Dropdown extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
        this.state = {
            open: false
        }
    }

    public componentWillUnmount() {
        window.removeEventListener("click", this.close)
    }

    private open = (e: React.MouseEvent<HTMLAnchorElement>): void => {
        if (this.state.open) {
            this.close();
            return;
        }

        e.stopPropagation();
        window.addEventListener("click", this.close)
        this.setState({ open: true });
    }

    private close = () => {
        window.removeEventListener("click", this.close)
        this.setState({ open: false });
    }

    public render() {
        return (
            <div className={this.props.className || "dropdown"}>
                <a href="#" className="btn" onClick={(e: React.MouseEvent<HTMLAnchorElement>) => this.open(e)}>
                    <i className="material-icons">{this.props.icon}</i>
                    {this.props.name}
                    {this.props.number === undefined ? null : <span>{this.props.number}</span>}
                    {this.props.showDownArrow ? <i className="material-icons">keyboard_arrow_down</i> : null}
                </a>
                {this.state.open ? <div className="dropdown-menu shadow">{this.props.children}</div> : null}
            </div>
        );
    }
}

interface IProps {
    name: string
    icon: string
    showDownArrow: boolean
    number?: number
    className?: string
}

interface IState {
    open: boolean
}

export const DropdownItem = (props: { name: string, icon: string, action(): void }): JSX.Element => {
    return (
        <a href="#" className="nav" onClick={props.action}>
            <i className="material-icons">{props.icon}</i> {props.name}
        </a>
    );
}