import * as React from 'react';
import { Link } from 'react-router-dom';

export default class Button extends React.Component<Props> {
    public constructor(props: Props) {
        super(props);
    }

    public render() {
        if ((this.props as IButton).action) {
            return <a href="#" className="btn" onClick={(this.props as IButton).action}>{this.props.name}</a>
        }

        return <Link className="btn" to={(this.props as ILink).to}>{this.props.name}</Link>
    }
}

interface IButton {
    name: string
    action(): void
}

interface ILink {
    name: string
    to: string
}

type Props = IButton | ILink;