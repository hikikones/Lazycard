import * as React from 'react';
import { NavLink } from "react-router-dom";

import db from '../model/Database';

export default class Nav extends React.Component {
    private add = (): void => {

    }

    public render() {
        const topics = db.topics.getAll();

        return (
            <nav>
                <NavSection title="Topics">
                    {topics.map(t => <NavItem key={t.id} name={t.name} to={`/topics/${t.id}`} />)}
                    <p>ADD</p>
                </NavSection>
            </nav>
        );
    }
}

class NavSection extends React.Component<INavSectionProps> {
    public constructor(props: INavSectionProps) {
        super(props);
    }

    public render() {
        return (
            <div>
                <h3>{this.props.title}</h3>
                {this.props.children}
            </div>
        )
    }
}

interface INavSectionProps {
    title: string
    children: React.ReactNode
}

class NavItem extends React.Component<INavSectionItem> {
    public constructor(props: INavSectionItem) {
        super(props);
    }

    public render() {
        return (
            <NavLink exact to={this.props.to}>{this.props.name}</NavLink>
        )
    }
}

interface INavSectionItem {
    name: string
    to: string
}