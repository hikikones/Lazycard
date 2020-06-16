import * as React from 'react';

import Keyboard from '../controller/Keyboard';

import Button from './Button';
import Slider from './Slider';
import Colors from './Colors';

const frontUndos = new Array<string>();
const frontRedos = new Array<string>();
const backUndos = new Array<string>();
const backRedos = new Array<string>();

let frontImageData: ImageData = null;
let backImageData: ImageData = null;
let width: number = window.innerWidth;
let height: number = window.innerHeight;
let showFront: boolean = true;

// TODO: also remember brush, size and color?
// TODO: implement save

const Canvas = () => {
    type Point = { x: number, y: number }
    const canvas = React.useRef<HTMLCanvasElement>(null);
    let context: CanvasRenderingContext2D;
    let mousePos: Point;
    let isDrawing: boolean = false;

    const [isFront, setIsFront] = React.useState<boolean>(showFront);
    const [brush, setBrush] = React.useState<string>("default");
    const [size, setSize] = React.useState<number>(5);
    const [color, setColor] = React.useState<string>("black");

    const handleDrawStart = (e: React.MouseEvent) => {
        addUndo();
        clearRedos();
        isDrawing = true;
        updateMousePos(e);
        applyBrush();
        drawCircle();
    }
    
    const handleDrawMove = (e: React.MouseEvent) => {
        if (!isDrawing) {
            updateMousePos(e);
            return;
        }
        const prevMousePos = mousePos;
        updateMousePos(e);
        drawLine(prevMousePos, mousePos);
    }

    const handleDrawEnd = (e: React.MouseEvent) => {
        isDrawing = false;
        context.globalCompositeOperation = "source-over";
    }

    const handleDrawEnter = (e: React.MouseEvent) => {
        if (e.buttons === 0) return;
        isDrawing = true;
        updateMousePos(e);
        applyBrush();
    }

    const updateMousePos = (e: React.MouseEvent) => {
        const x = e.pageX - canvas.current.offsetLeft;
        const y = e.pageY - canvas.current.offsetTop;
        mousePos = { x, y };
    }

    const drawLine = (from: Point, to: Point) => {
        context.beginPath();
        context.moveTo(from.x, from.y);
        context.lineTo(to.x, to.y)
        context.closePath();
        context.stroke();
    }

    const drawCircle = () => {
        context.beginPath();
        context.arc(mousePos.x, mousePos.y, context.lineWidth / 2, 0, 2 * Math.PI, false);
        context.closePath();
        context.fill();
    }

    const applyBrush = () => {
        context.lineJoin = "round";
        context.lineCap = "round";
        context.fillStyle = color;
        context.strokeStyle = color;
        context.lineWidth = size;
        
        switch(brush) {
            case "eraser": {
                context.globalCompositeOperation = "destination-out";
                context.fillStyle = "#ff0000";
                context.strokeStyle= "#ff0000";
                break;
            }
            default: {
                context.globalCompositeOperation = "source-over";
                break;
            }
        }
    }

    const flip = () => {
        setImageData();
        showFront = !showFront;
        clear();
        const otherSide = getImageData();
        if (otherSide !== null) replaceDrawing(otherSide);
        setIsFront(showFront);
    }

    const undo = () => {
        if (getUndos().length === 0) return;
        addRedo();
        const img = new Image();
        img.src = getUndos().pop();
        img.onload = () => {
            clear();
            context.drawImage(img, 0, 0);
        }
    }

    const redo = () => {
        if (getRedos().length === 0) return;
        addUndo();
        const img = new Image();
        img.src = getRedos().pop();
        img.onload = () => {
            clear();
            context.drawImage(img, 0, 0);
        }
    }

    const clear = () => {
        context.clearRect(0, 0, context.canvas.width, context.canvas.height);
    }

    const getImageData = (): ImageData => {
        if (showFront) return frontImageData;
        return backImageData;
    }

    const setImageData = () => {
        if (showFront) {
            frontImageData = getCanvasImageData();
        } else {
            backImageData = getCanvasImageData();
        }
    }

    const getCanvasImageData = (): ImageData => {
        return context.getImageData(0, 0, canvas.current.width, canvas.current.height);
    }

    const replaceDrawing = (imageData: ImageData) => {
        context.putImageData(imageData, 0, 0);
    }

    const snapshot = (): string => {
        return canvas.current.toDataURL();
    }

    const onImagePaste = (e: ClipboardEvent) => {
        for (let i = 0 ; i < e.clipboardData.items.length; i++) {
            const item = e.clipboardData.items[i];
            if (item.type.indexOf("image") === -1) continue;
            addUndo();
            const blob = item.getAsFile();
            const img = new Image();
            img.src = (window.URL || window.webkitURL).createObjectURL(blob);
            img.onload = () => context.drawImage(img, mousePos.x - img.width/2, mousePos.y - img.height/2);
        }
    }

    const resize = () => {
        if (window.innerWidth < canvas.current.width || window.innerHeight < canvas.current.height) return;
        const imageData = getCanvasImageData();
        width = document.body.clientWidth;
        height = document.body.clientHeight;
        canvas.current.width = width;
        canvas.current.height = height;
        replaceDrawing(imageData);
    }

    const crop = (): ImageData => {
        const w = window.innerWidth;
        const h = window.innerHeight;
        const min = { x: Number.MAX_SAFE_INTEGER, y: Number.MAX_SAFE_INTEGER };
        const max = { x: Number.MIN_SAFE_INTEGER, y: Number.MIN_SAFE_INTEGER };
        const imageData = context.getImageData(0, 0, w, h);
        
        for (let y = 0; y < h; y++) {
            for (let x = 0; x < w; x++) {
                const index = (y * w + x) * 4;
                const alpha = imageData.data[index + 3];
                if (alpha > 0) {
                    min.x = Math.min(min.x, x);
                    max.x = Math.max(max.x, x);
                    min.y = Math.min(min.y, y);
                    max.y = Math.max(max.y, y);
                }
            }
        }

        if (min.x === Number.MAX_SAFE_INTEGER || max.x === Number.MIN_SAFE_INTEGER) {
            return context.getImageData(0, 0, w, h);
        }

        const newWidth = max.x - min.x + 1;
        const newHeight = max.y - min.y + 1;
        return context.getImageData(min.x, min.y, newWidth, newHeight);
    }

    const toDataURL = (): string => {
        const cvs: HTMLCanvasElement = document.createElement("canvas");
        const ctx: CanvasRenderingContext2D = cvs.getContext("2d");
        const img: ImageData = crop();
        cvs.width = img.width;
        cvs.height = img.height;
        ctx.putImageData(img, 0, 0);
        return cvs.toDataURL();
    }

    const addUndo = () => {
        getUndos().push(snapshot());
    }

    const addRedo = () => {
        getRedos().push(snapshot());
    }

    const clearRedos = () => {
        getRedos().length = 0;
    }

    const getUndos = (): string[] => {
        if (showFront) return frontUndos;
        return backUndos;
    }

    const getRedos = (): string[] => {
        if (showFront) return frontRedos;
        return backRedos;
    }

    React.useLayoutEffect(() => {
        context = canvas.current.getContext("2d");
        document.onpaste = (e: ClipboardEvent) => onImagePaste(e);
        return () => {
            setImageData();
        }
    });

    React.useLayoutEffect(() => {
        resize();
        window.onresize = () => resize();
        const imageData = getImageData();
        if (imageData !== null) replaceDrawing(getImageData());
        return () => {
            window.onresize = null;
            document.onpaste = null;
        }
    }, []);

    return (
        <div>
            <canvas
                ref={canvas}
                onMouseDown={(e: React.MouseEvent) => handleDrawStart(e)}
                onMouseMove={(e: React.MouseEvent) => handleDrawMove(e)}
                onMouseUp={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseLeave={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseEnter={(e: React.MouseEvent) => handleDrawEnter(e)}
            />
            <div className="draw-menu row-of-items">
                <Button
                    icon="done"
                    action={() => console.log(toDataURL())}
                />
                <Button
                    icon="flip_to_back"
                    icon2="flip_to_front"
                    toggle={showFront}
                    action={flip}
                    shortcut={Keyboard.Space}
                />
                <Button
                    icon="undo"
                    action={undo}
                />
                <Button
                    icon="redo"
                    action={redo}
                />
                <Button
                    icon="create"
                    action={() => setBrush("default")}
                />
                <Button
                    icon="clear"
                    action={() => setBrush("eraser")}
                />
                <Slider onChange={(val: number) => setSize(val)} defaultSize={size} />
            </div>
            <Colors onColorPick={(col: string) => setColor(col)} />
        </div>
    );
}

export default Canvas;