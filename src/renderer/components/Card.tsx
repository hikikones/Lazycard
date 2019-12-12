import * as React from 'react';

export default class Card extends React.Component<IProps> {
    public constructor(props: IProps) {
        super(props);
    }

    // TODO: Markdown.

    public render() {
        return (
            <div>
                {this.props.front ? <p>{this.props.front}</p> : null}
                {this.props.back ? <hr /> : null}
                {this.props.back ? <p>{this.props.back}</p> : null}
            </div>
        );
    }
}

interface IProps {
    front?: string
    back?: string
}