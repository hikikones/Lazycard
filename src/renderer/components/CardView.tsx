import * as React from 'react';

import md from '../controller/Markdown';

const Card = (props: ICardViewProps) => {
    return (
        <div className="card shadow">
            <CardContent markdown={props.front} />
            {props.showBack && <hr />}
            {props.showBack && <CardContent markdown={props.back} />}

            {props.children || null}
        </div>
    );
}

interface ICardViewProps {
    front: string
    back: string
    showBack: boolean
    children?: React.ReactNode
}

const CardContent = (props: { markdown: string }) => {
    return <div dangerouslySetInnerHTML={{ __html: md.parse(props.markdown) }} />
}

export default Card;