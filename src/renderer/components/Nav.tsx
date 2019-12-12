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
        // TODO: db.import?
        console.log("IMPORT");
    }

    public render() {
        return (
            <nav>
                <NavSection title="Cards">
                    <NavSectionLink name="Home" to="/" />
                    <NavSectionLink name="Due today" to="/review/:topicId" />
                    <NavSectionLink name="All cards" to="/cards/" />
                </NavSection>

                <NavSection title="Database">
                    <NavSectionLink name="Settings" to="/settings" />
                    <NavSectionButton name="Import" action={this.import} />
                </NavSection>

                <NavSection title="Topics">
                    {this.props.topics.map(t =>
                        <NavSectionLink key={t.id} name={t.name} to={`/topics/${t.id}`} />
                    )}
                    <NavSectionButton name="Add topic" action={this.addTopic} />
                </NavSection>
            </nav>
        );
    }
}

interface INavProps {
    topics: readonly Topic[]
    onTopicChange(): void
}

const NavSection = (props: { title: string, children: React.ReactNode }) => {
    return (
        <section>
            <h4>{props.title}</h4>
            <ul>
                {props.children}
            </ul>
        </section>
    )
}

const NavSectionLink = (props: { name: string, to: string }) => {
    return (
        <li>
            <NavLink exact to={props.to}>{props.name}</NavLink>
        </li>
    )
}

const NavSectionButton = (props: { name: string, action(): void }) => {
    return (
        <li>
            <a href="#" onClick={props.action}>{props.name}</a>
        </li>
    )
}