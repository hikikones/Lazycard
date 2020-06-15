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
                onSave={save}
                onFlip={() => setShowFront(!showFront)}
                onUndo={() => showFront ? front.current.undo() : back.current.undo()}
                onRedo={() => showFront ? front.current.redo() : back.current.redo()}
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
            <a onClick={props.onSave}>
                <i className="material-icons">done</i>
            </a>
            <a onClick={flip}>
                <i className="material-icons">{isFlipped ? "flip_to_front" : "flip_to_back"}</i>
            </a>
            <a onClick={props.onUndo}>
                <i className="material-icons">undo</i>
            </a>
            <a onClick={props.onRedo}>
                <i className="material-icons">redo</i>
            </a>
        </div>
    );
}

interface IDrawMenuProps {
    onFlip(): void
    onSave(): void
    onUndo(): void
    onRedo(): void
}

export default Draw;