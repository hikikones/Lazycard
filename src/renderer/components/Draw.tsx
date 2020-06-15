import * as React from 'react';
import { RouteComponentProps } from "react-router-dom";

import db from '../model/Database';

import Colors from './Colors';
import Canvas from './Canvas';
import Button from './Button';

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
            <div className="draw-menu">
                <Button
                    icon="done"
                    action={save}
                />
                <Button
                    icon={showFront ? "flip_to_back" : "flip_to_front"}
                    action={() => setShowFront(!showFront)}
                />
                <Button
                    icon="undo"
                    action={() => showFront ? front.current.undo() : back.current.undo()}
                />
                <Button
                    icon="redo"
                    action={() => showFront ? front.current.redo() : back.current.redo()}
                />
            </div>
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

export default Draw;