import * as React from 'react';

import Button from './Button';

const colors = ["black", "blue", "red", "yellow", "pink"];

const Colors = (props: {onColorPick(color: string): void}) => {
    return (
        <div className="colors-menu">
            {colors.map((c, i) =>
                <Button
                    icon="lens"
                    action={() => props.onColorPick(c)}
                    color={c}
                />
            )}
        </div>
    );
}

export default Colors;