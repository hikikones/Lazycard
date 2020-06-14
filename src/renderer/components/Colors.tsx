import * as React from 'react';

const colors = ["black", "blue", "red", "yellow", "pink"];

const Colors = (props: {onColorPick(color: string): void}) => {
    return (
        <div className="colors-menu">
            {colors.map((c, i) =>
                <Color color={c} onPick={() => props.onColorPick(c)} key={i} />
            )}
        </div>
    );
}

const Color = (props: {color: string, onPick(color: string): void}) => {
    return (
        <a
            style={{backgroundColor: props.color}}
            onClick={() => props.onPick(props.color)}
        />
    );
}

export default Colors;