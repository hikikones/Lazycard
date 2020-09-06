import * as React from 'react';

import md from '../controller/Markdown';

const Card = (props: ICardViewProps) => {
    return (
        <div className = "card-padding">
        <div className="card card-padding">
            <CardContent markdown={props.front} />
            {props.children || null}
        </div>
        </div>
    );
}

interface ICardViewProps {
    front: string
    children?: React.ReactNode
}

const CardContent = (props: { markdown: string }) => {
    return <div dangerouslySetInnerHTML={{ __html: md.parse(props.markdown) }} />
}

export default Card;