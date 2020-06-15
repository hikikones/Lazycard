import * as React from 'react';
import { RouteComponentProps } from "react-router-dom";

import Canvas from './Canvas';

const Draw = (props: IDrawProps) => {
    const id = parseInt(props.match.params.id);

    return (
        <Canvas />
    );
}

interface IDrawProps extends RouteComponentProps<{ id: string }> {}

export default Draw;