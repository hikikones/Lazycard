import * as React from 'react';

const App = () => {
    const canvas = React.useRef<HTMLCanvasElement>(null);
    let context: CanvasRenderingContext2D;
    type Point = { x: number, y: number }
    let mousePos: Point;
    let isDrawing: boolean = false;
    let undos: string[] = new Array<string>();
    let redos: string[] = new Array<string>();

    const handleDrawStart = (e: React.MouseEvent) => {
        undos.push(canvas.current.toDataURL());
        redos = new Array<string>();
        isDrawing = true;
        updateMousePos(e);
        context.lineJoin = "round";
        context.lineCap = "round";
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

    React.useEffect(() => {
        context = canvas.current.getContext("2d");
        window.addEventListener("paste", (e: ClipboardEvent) => {onImagePaste(e)});
        return () => {
            window.removeEventListener("paste", (e: ClipboardEvent) => {onImagePaste(e)});
        }
    });

    return (
        <div>
            <canvas
                ref={canvas} width={500} height={500}
                onMouseDown={(e: React.MouseEvent) => handleDrawStart(e)}
                onMouseMove={(e: React.MouseEvent) => handleDrawMove(e)}
                onMouseUp={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseLeave={(e: React.MouseEvent) => handleDrawEnd(e)}
                onMouseEnter={(e: React.MouseEvent) => handleDrawEnter(e)}
            />
            <button onClick={undo}>Undo</button>
            <button onClick={redo}>Redo</button>
        </div>
    );
}

export default App;