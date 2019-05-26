import React from 'react';
import { Link } from 'react-router-dom';

export default class Header extends React.Component {
    constructor(props) {
        super(props);
        this.onSearchSubmit = this.onSearchSubmit.bind(this);
        this.state = { text: null }
    }

    onSearchSubmit(event) {
        if (event.key !== "Enter") {
            return;
        }
        const textFormatted = event.target.value.replace(/[&\/\\#,+()$~%.'":*?<>{}]/g, '');
        this.setState({ text: textFormatted }, () => this.redirect());
    }

    redirect() {
        document.getElementById("link").click();
    }

    render() {
        return (
            <header>
                <Link id="link" to={`/search/${this.state.text}`}></Link>
                <input
                className="search"
                type="text"
                placeholder="Search..."
                onKeyDown={event => this.onSearchSubmit(event)} />
            </header>
        );
    }
}
