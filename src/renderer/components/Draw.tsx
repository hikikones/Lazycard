import * as React from 'react';
import { RouteComponentProps } from "react-router-dom";

import db from '../model/Database';

import Nav from './Nav';
import Canvas from './Canvas';

const Draw = (props: IProps) => {
    const id = parseInt(props.match.params.id);
    const [showFront, setShowFront] = React.useState<boolean>(true);
    const [color, setColor] = React.useState<string>("black");

    const front = React.useRef(null);
    const back = React.useRef(null);

    const save = () => {
        const card = id ? db.cards.get(id) : db.cards.new();
        card.front = front.current.toDataURL();
        card.back = back.current.toDataURL();
    }

    return (
        <div>
            <Nav id={id} />
            
            <Canvas
                ref={front}
                show={showFront}
                color={color}
            />
            <Canvas
                ref={back}
                show={!showFront}
                color={color}
            />

            <div>
                <button onClick={() => setColor("blue")}>Blue</button>
                <button onClick={() => setShowFront(!showFront)}>Flip</button>
                <button onClick={save}>SAVE</button>
            </div>
        </div>
    );
}

interface IProps extends RouteComponentProps<{ id: string }> {}

export default Draw;