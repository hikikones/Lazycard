import * as React from 'react';

import { Card } from '../model/Database';

import Button from './Button';

export default class CardMenu extends React.Component<IProps, IState> {
    public constructor(props: IProps) {
        super(props);
        this.state = {
            showMenu: false
        }
    }

    private toggleMenu = () => {
        this.setState({ showMenu: !this.state.showMenu });
    }

    private onEdit = () => {
        this.toggleMenu();
        this.props.onEdit(this.props.card);
    }

    private onDelete = () => {
        this.toggleMenu();
        this.props.onDelete(this.props.card);
    }

    public render() {
        return (
            <div className="card-btn">
                <Button name="" icon="more_horiz" action={this.toggleMenu} />
                {this.state.showMenu
                    ?   <div className="card-menu">
                            <a href="#" onClick={this.onEdit} className="nav">
                                <i className="material-icons">edit</i> Edit
                            </a>
                            <a href="#" onClick={this.onDelete} className="nav">
                                <i className="material-icons">delete</i> Delete
                            </a>
                        </div>
                    :   null
                }
            </div>
        );
    }
}

interface IProps {
    card: Card
    onEdit(card: Card): void
    onDelete(card: Card): void
}

interface IState {
    showMenu: boolean
}