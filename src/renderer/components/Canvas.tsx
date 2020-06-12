import * as React from 'react';

const Canvas = React.forwardRef<Handles, IProps>((props, ref) => {
    type Point = { x: number, y: number }
    const canvas = React.useRef<HTMLCanvasElement>(null);
    let context: CanvasRenderingContext2D;
    let mousePos: Point;
    let isDrawing: boolean = false;

    const [undos] = React.useState<Array<string>>([]);
    const [redos] = React.useState<Array<string>>([]);

    const handleDrawStart = (e: React.MouseEvent) => {
        undos.push(canvas.current.toDataURL());
        redos.length = 0;
        isDrawing = true;
        updateMousePos(e);
        context.lineJoin = "round";
        context.lineCap = "round";
        context.fillStyle = props.color;
        context.strokeStyle = props.color;
        context.lineWidth = 5;
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
    }

    const handleDrawEnter = (e: React.MouseEvent) => {
        if (e.buttons === 0) return;
        isDrawing = true;
        updateMousePos(e);
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

    const onImagePaste = (e: ClipboardEvent) => {
        for (let i = 0 ; i < e.clipboardData.items.length; i++) {
            const item = e.clipboardData.items[i];
            if (item.type.indexOf("image") === -1) continue;
            undos.push(canvas.current.toDataURL());
            const blob = item.getAsFile();
            const img = new Image();
            img.src = (window.URL || window.webkitURL).createObjectURL(blob);
            img.onload = () => context.drawImage(img, mousePos.x - img.width/2, mousePos.y - img.height/2);
        }
    }

    const undo = () => {
        if (undos.length === 0) return;
        redos.push(canvas.current.toDataURL());
        const img = new Image();
        img.src = undos.pop();
        img.onload = () => {
            clear();
            context.drawImage(img, 0, 0);
        }
    }

    const redo = () => {
        if (redos.length === 0) return;
        undos.push(canvas.current.toDataURL());
        const img = new Image();
        img.src = redos.pop();
        img.onload = () => {
            clear();
            context.drawImage(img, 0, 0);
        }
    }

    const clear = () => {
        context.clearRect(0, 0, context.canvas.width, context.canvas.height);
    }

    const resize = () => {
        if (window.innerWidth < canvas.current.width || window.innerHeight < canvas.current.height) return;
        const imageData = context.getImageData(0, 0, canvas.current.width, canvas.current.height);
        canvas.current.width = document.body.clientWidth;
        canvas.current.height = document.body.clientHeight;
        context.putImageData(imageData, 0, 0);
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

    React.useEffect(() => {
        context = canvas.current.getContext("2d");
        window.addEventListener("resize", resize);
        document.onpaste = (e: ClipboardEvent) => onImagePaste(e);
    });

    React.useImperativeHandle(ref, () => ({
        toDataURL: (): string => {
            return toDataURL();
        }
    }));

    return (
        <div className={props.show ? "" : "hide"}>
            <canvas
                ref={canvas}
                onMouseDown={(e: React.MouseEvent) => handleDrawStart(e)}
                onMouseMove={(e: React.MouseEvent) => handleDrawMove(e)}
                onMouseUp={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseLeave={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseEnter={(e: React.MouseEvent) => handleDrawEnter(e)}
            />
            <button onClick={undo}>Undo</button>
            <button onClick={redo}>Redo</button>
            <button onClick={() => console.log(crop())}>Crop</button>
            <button onClick={() => console.log(toDataURL())}>toDataURL</button>
        </div>
    );
});

interface IProps {
    show: boolean
    color: string
}

interface Handles {
    toDataURL(): string
}

export default Canvas;