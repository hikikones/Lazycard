import * as React from 'react';

const Slider = (props: ISliderProps) => {
    return (
        <input
            className="slider button"
            type="range"
            defaultValue={props.defaultSize}
            min={1}
            max={64}
            onInput={(e: React.FormEvent<HTMLInputElement>) => props.onChange(Number(e.currentTarget.value))}
        />
    );
}

interface ISliderProps {
    defaultSize: number
    onChange(newValue: number): void
}

export default Slider;