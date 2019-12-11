import * as React from 'react';
import { NavLink } from "react-router-dom";

import db from '../model/Database';
import { Topic } from '../model/Database';

export default class Nav extends React.Component<INavProps> {
    public constructor(props: INavProps) {
        super(props);
    }

    private addTopic = (): void => {
        const topic: Topic = new Topic("New Topic");
        db.topics.add(topic);
        this.props.onTopicChange();
    }

    public render() {
        return (
            <nav>
                <NavSection title="Topics">
                    {this.props.topics.map(t => <NavItem key={t.id} name={t.name} to={`/topics/${t.id}`} />)}
                    <button onClick={this.addTopic}>Add</button>
                </NavSection>
            </nav>
        );
    }
}

interface INavProps {
    topics: readonly Topic[]
    onTopicChange(): void
}

class NavSection extends React.Component<INavSectionProps> {
    public constructor(props: INavSectionProps) {
        super(props);
    }

    public render() {
        return (
            <section>
                <h4>{this.props.title}</h4>
                <ul>
                    {this.props.children}
                </ul>
            </section>
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
            <li>
                <NavLink exact to={this.props.to}>{this.props.name}</NavLink>
            </li>
        )
    }
}

interface INavSectionItem {
    name: string
    to: string
}