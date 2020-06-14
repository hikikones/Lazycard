import * as React from 'react';
import { RouteComponentProps } from "react-router-dom";

import db from '../model/Database';

import Colors from './Colors';
import Canvas from './Canvas';

const Draw = (props: IDrawProps) => {
    const id = parseInt(props.match.params.id);
    const [showFront, setShowFront] = React.useState<boolean>(true);
    const [color, setColor] = React.useState<string>("black");

    const front = React.useRef(null);
    const back = React.useRef(null);

    const onColorPick = (color: string) => {
        setColor(color);
    }

    const save = () => {
        const card = id ? db.cards.get(id) : db.cards.new();
        card.front = front.current.toDataURL();
        card.back = back.current.toDataURL();
    }

    return (
        <div>
            <Menu
                onFlip={() => setShowFront(!showFront)}
                onSave={save}
            />
            <Colors onColorPick={onColorPick} />

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
        </div>
    );
}

interface IDrawProps extends RouteComponentProps<{ id: string }> {}

const Menu = (props: IDrawMenuProps) => {
    const [isFlipped, setIsFlipped] = React.useState<boolean>(false);

    const flip = () => {
        setIsFlipped(!isFlipped);
        props.onFlip();
    }

    return (
        <div className="draw-menu">
            <a onClick={flip}>
                <i className="material-icons">{isFlipped ? "flip_to_front" : "flip_to_back"}</i>
            </a>
            <a onClick={props.onSave}>
                <i className="material-icons">done</i>
            </a>
        </div>
    );
}

interface IDrawMenuProps {
    onFlip(): void
    onSave(): void
}

export default Draw;