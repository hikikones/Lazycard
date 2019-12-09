import * as React from 'react';

export default class Card extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
    }

    public render() {
        return (
            <div>
                <p>{this.props.front}</p>
                <p>{this.props.back}</p>
            </div>
        );
    }
}

interface IProps {
    front?: string
    back?: string
}

interface IState {
    //name: string
}