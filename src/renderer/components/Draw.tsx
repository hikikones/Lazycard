import * as React from 'react';
import { NavLink, RouteComponentProps } from "react-router-dom";

import Canvas from './Canvas';

const Draw = (props: IProps) => {
    const id = parseInt(props.match.params.id);
    const [front, setFront] = React.useState<boolean>(true);
    const [color, setColor] = React.useState<string>("black");

    return (
        <div>
            <nav>
                <NavLink exact to="/">
                    <button>Review</button>
                </NavLink>
                <button>{id ? "Save" : "Add"}</button>
            </nav>
            <Canvas
                show={front}
                color={color}
            />
            <Canvas
                show={!front}
                color={color}
            />
            <div>
                <button onClick={() => setColor("blue")}>Blue</button>
                <button onClick={() => setFront(!front)}>Flip</button>
            </div>
        </div>
    );
}

interface IProps extends RouteComponentProps<{ id: string }> {}

export default Draw;