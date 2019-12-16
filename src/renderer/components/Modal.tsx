import * as React from 'react';

export default class App extends React.Component<IProps> {
    public constructor(props: IProps) {
        super(props);
    }

    private onClickOutside = () => {
        this.props.onClickOutside();
    }

    public render() {
        if (!this.props.show) return null;

        return (
            <div className='modal' onClick={this.onClickOutside}>
                <div onClick={(e: React.MouseEvent<HTMLDivElement>) => e.stopPropagation()}>
                    {this.props.children}
                </div>
            </div>
        );
    }
}

interface IProps {
    show: boolean
    onClickOutside(): void
}