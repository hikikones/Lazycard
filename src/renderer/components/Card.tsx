import * as React from 'react';

import md from '../controller/Markdown';

export default class Card extends React.Component<IProps> {
    public constructor(props: IProps) {
        super(props);
    }

    public render() {
        return (
            <div>
                {this.props.front === undefined ? null : <CardContent markdown={this.props.front} />}
                {this.props.back === undefined ? null : <hr />}
                {this.props.back === undefined ? null : <CardContent markdown={this.props.back} />}
                {this.props.children}
            </div>
        );
    }
}

interface IProps {
    front?: string
    back?: string
}

const CardContent = (props: { markdown: string }) => {
    return <div dangerouslySetInnerHTML={{ __html: md.parse(props.markdown) }} />
}