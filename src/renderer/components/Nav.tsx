import * as React from 'react';
import { NavLink } from "react-router-dom";

import db from '../model/Database';
import { Topic } from '../model/Database';

export default class Nav extends React.Component<INavProps> {
    public constructor(props: INavProps) {
        super(props);
    }

    private addTopic = (): void => {
        const topic: Topic = db.topics.new("New Topic");
        this.props.onTopicChange();
    }

    private import = () => {
        db.import();
        this.props.onTopicChange();
    }

    public render() {
        return (
            <nav>
                <NavItem name="Home" to="/" />
                <NavItem name="Due today" to="/review/:topicId" />
                <NavItem name="All cards" to="/cards/" />
                <NavItem name="Settings" to="/settings" />
                <NavItem name="Import" action={this.import} />

                <label>Topics</label>
                {this.props.topics.map(t =>
                    <NavItem key={t.id} name={t.name} to={`/topics/${t.id}`} />
                )}
                <NavItem name="Add topic" action={this.addTopic} />
            </nav>
        );
    }
}

interface INavProps {
    topics: readonly Topic[]
    onTopicChange(): void
}

const NavItem = (props: INavLink | INavButton) => {
    if ((props as INavLink).to) {
        return <NavLink className="nav" exact to={(props as INavLink).to}>{props.name}</NavLink>
    }

    return <a href="#" className="nav" onClick={(props as INavButton).action}>{props.name}</a>
}

interface INavLink {
    name: string
    to: string
}

interface INavButton {
    name: string
    action(): void
}