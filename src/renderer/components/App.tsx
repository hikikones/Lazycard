import * as React from 'react';

import Canvas from './Canvas';

const App = () => {
    const [front, setFront] = React.useState<boolean>(true);
    const [color, setColor] = React.useState<string>("black");

    return (
        <div>
            <Canvas
                show={front}
                color={color}
            />
            <Canvas
                show={!front}
                color={color}
            />
            <button onClick={() => setColor("blue")}>Blue</button>
            <button onClick={() => setFront(!front)}>Flip</button>
        </div>
    );
}

export default App;