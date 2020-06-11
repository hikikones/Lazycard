import * as React from 'react';
import { NavLink } from "react-router-dom";

const Review = () => {
    const [id, setId] = React.useState<number>(1);

    return (
        <div>
            <nav>
                <NavLink to="/draw/0">
                    <button>New</button>
                </NavLink>
                <NavLink to={`/draw/${id}`}>
                    <button>Edit</button>
                </NavLink>
            </nav>
            <h1>Review</h1>
        </div>
    );
}

export default Review;