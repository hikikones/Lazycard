import * as React from 'react';

import ButtonNavLink from './ButtonNavLink';

const Nav = () => {
    return (
        <nav className="col space-between full-height">
            <div>
                <ButtonNavLink to="/review/0" icon="drafts" routeName="review" className="navigation" />
                <ButtonNavLink to="/topics" icon="dashboard" className="navigation" />
            </div>
            <div>
                <ButtonNavLink to="/settings" icon="settings" className="navigation" />
            </div>
        </nav>
    );
}

export default Nav;