import * as React from 'react';

const App = () => {
    const canvas = React.useRef<HTMLCanvasElement>(null);
    type Position = { x: number, y: number }
    let mousePos: Position;
    let isDrawing: boolean = false;

    const handleDrawMove = (e: React.MouseEvent) => {
        if (!isDrawing) {
            updateMousePos(e);
            return;
        }
        const oldPos = mousePos;
        updateMousePos(e);
        drawLine(oldPos, mousePos);
    }

    const handleDrawStart = (e: React.MouseEvent) => {
        isDrawing = true;
        updateMousePos(e);
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

    const drawLine = (from: Position, to: Position) => {
        const ctx: CanvasRenderingContext2D = canvas.current.getContext("2d");
        ctx.beginPath();
        ctx.lineJoin = "round";
        ctx.lineWidth = 5;
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y)
        ctx.closePath();
        ctx.stroke();
    }

    const drawCircle = () => {
        const ctx: CanvasRenderingContext2D = canvas.current.getContext("2d");
        ctx.beginPath();
        ctx.arc(mousePos.x, mousePos.y, 5, 0, 2*Math.PI, false);
        ctx.closePath();
        ctx.fill();
    }

    const onImagePaste = (e: ClipboardEvent) => {
        for (let i = 0 ; i < e.clipboardData.items.length; i++) {
            const item = e.clipboardData.items[i];
            if (item.type.indexOf("image") == -1) continue;
            const blob = item.getAsFile();
            const ctx: CanvasRenderingContext2D = canvas.current.getContext("2d");
            const img = new Image();
            img.onload = () => {
                ctx.drawImage(img, mousePos.x - img.width/2, mousePos.y - img.height/2);
            }
            img.src = (window.URL || window.webkitURL).createObjectURL(blob);
        }
    }

    React.useEffect(() => {
        window.addEventListener("paste", (e: ClipboardEvent) => {onImagePaste(e)});
        return () => {
            window.removeEventListener("paste", (e: ClipboardEvent) => {onImagePaste(e)});
        }
    });

    return (
        <canvas
            ref={canvas} width={500} height={500}
            onMouseDown={(e: React.MouseEvent) => handleDrawStart(e)}
            onMouseMove={(e: React.MouseEvent) => handleDrawMove(e)}
            onMouseUp={(e: React.MouseEvent) => handleDrawEnd(e)}
            onMouseLeave={(e: React.MouseEvent) => handleDrawEnd(e)}
            onMouseEnter={(e: React.MouseEvent) => handleDrawEnter(e)}
            //onClick={() => drawCircle()}
        />
    );
}

export default App;