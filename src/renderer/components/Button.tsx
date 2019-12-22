import * as React from 'react';
import { Link } from 'react-router-dom';

export default class Button extends React.Component<Props> {
    public constructor(props: Props) {
        super(props);
    }

    public click = () => {
        if ((this.props as IButton).action) {
            (this.props as IButton).action();
        }
    }

    public render() {
        if ((this.props as IButton).action) {
            return (
                <a href="#" className="btn" onClick={(this.props as IButton).action}>
                    <i className="material-icons">{this.props.icon}</i> {this.props.name}
                </a>
            );
        }

        return (
            <Link className="btn" to={(this.props as ILink).to}>
                <i className="material-icons">{this.props.icon}</i> {this.props.name}
            </Link>
        );
    }
}

interface IButton {
    icon: string
    name: string
    action(): void
}

interface ILink {
    icon: string
    name: string
    to: string
}

type Props = IButton | ILink;