import * as React from 'react';

import md from '../controller/Markdown';

const Card = (props: ICardViewProps) => {
    return (
        <div className="card shadow">
            <CardContent front={props.front} time={props.time} />
            {props.children || null}
        </div>
    );
}

interface ICardViewProps {
    front: string
    time: number
    children?: React.ReactNode
}

interface ICardContentProps {
    front: string
    time: number
}

const CardContent = (props: ICardContentProps) => {
    const htmlString = props.front;
    return <div dangerouslySetInnerHTML={{ __html: md.parse(htmlString) }} />
}


export default Card;