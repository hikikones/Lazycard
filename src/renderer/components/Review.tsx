import * as React from 'react';
import { NavLink } from "react-router-dom";

import Nav from './Nav';

const Review = () => {
    const [id, setId] = React.useState<number>(1);

    return (
        <div>
            <Nav id={id} />
            <h1>Review</h1>
        </div>
    );
}

export default Review;