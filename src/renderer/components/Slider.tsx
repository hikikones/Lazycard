import * as React from 'react';

const Slider = (props: ISliderProps) => {
    const [size, setSize] = React.useState<number>(props.size);

    const min = 1;
    const max = 64;

    const onWheel = (e: WheelEvent) => {
        const forwards = e.deltaY < 0;
        const increment = forwards ? 1 : -1;
        setSize(s => clamp(s + increment));
    }

    const clamp = (val: number): number => {
        if (val < min) return min;
        if (val > max) return max;
        return val;
    }

    if (props.enableShortcut) {
        React.useEffect(() => {
            window.addEventListener("wheel", onWheel);
            props.onChange(size);
            return () => {
                window.removeEventListener("wheel", onWheel);
            }
        }, [size]);
    }

    return (
        <input
            className="slider button"
            type="range"
            value={size}
            onChange={(e: React.FormEvent<HTMLInputElement>) => setSize(Number(e.currentTarget.value))}
            min={min}
            max={max}
            onInput={(e: React.FormEvent<HTMLInputElement>) => props.onChange(Number(e.currentTarget.value))}
        />
    );
}

interface ISliderProps {
    size: number
    onChange(newValue: number): void
    enableShortcut: boolean
}

export default Slider;