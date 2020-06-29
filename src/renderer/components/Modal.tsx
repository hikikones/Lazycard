import * as React from 'react';

const Modal = (props: IModalProps) => {
    if (!props.show)
        return null;

    return (
        <div className="modal col col-center" onClick={props.onClickOutside}>
            <div onClick={(e: React.MouseEvent<HTMLDivElement>) => e.stopPropagation()}>
                {props.children}
            </div>
        </div>
    );
}

interface IModalProps {
    show: boolean
    onClickOutside(): void
    children: React.ReactNode
}

export default Modal;