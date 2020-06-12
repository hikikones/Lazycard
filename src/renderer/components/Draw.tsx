import * as React from 'react';
import { RouteComponentProps } from "react-router-dom";

import Nav from './Nav';
import Canvas from './Canvas';

const Draw = (props: IProps) => {
    const id = parseInt(props.match.params.id);
    const [showFront, setShowFront] = React.useState<boolean>(true);
    const [color, setColor] = React.useState<string>("black");

    return (
        <div>
            <Nav id={id} />
            
            <Canvas
                show={showFront}
                color={color}
            />
            <Canvas
                show={!showFront}
                color={color}
            />

            <div>
                <button onClick={() => setColor("blue")}>Blue</button>
                <button onClick={() => setShowFront(!showFront)}>Flip</button>
            </div>
        </div>
    );
}

interface IProps extends RouteComponentProps<{ id: string }> {}

export default Draw;