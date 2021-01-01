import * as React from "react";

import ButtonNavLink from "./ButtonNavLink";

const Nav = () => {
  return (
    <nav className="col space-between full-height">
      <div>
        <ButtonNavLink
          id="reviews-nav"
          to="/review/0"
          icon="drafts"
          routeName="review"
          className="navigation"
        />
        <ButtonNavLink
          id="cards-nav"
          to="/cards"
          icon="layers"
          className="navigation"
        />
        <ButtonNavLink
          id="topics-nav"
          to="/topics"
          icon="dashboard"
          className="navigation"
        />
      </div>
      <div>
        <ButtonNavLink to="/settings" icon="settings" className="navigation" />
      </div>
    </nav>
  );
};

export default Nav;
