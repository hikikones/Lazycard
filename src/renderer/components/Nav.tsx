import * as React from 'react';
import { NavLink } from "react-router-dom";

const Nav = (props: IProps) => {
    return (
        <div>
            <nav>
                <NavLink to="/">
                    <button>Review</button>
                </NavLink>
                <NavLink to={`/draw/${props.id}`}>
                    <button>{props.id ? "Edit" : "Add"}</button>
                </NavLink>
                <NavLink to="/settings">
                    <button>Settings</button>
                </NavLink>
            </nav>
        </div>
    );
}

interface IProps {
    id: number
}

export default Nav;