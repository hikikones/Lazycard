import * as React from 'react';

const Empty = (props: IEmptyProps) => {
    return (
        <section className="col col-center row-center full-height empty-state">
            <i className="material-icons icon">{props.icon}</i>
            <h3>{props.message}</h3>
        </section>
    );
}

interface IEmptyProps {
    icon: string
    message: string
}

export default Empty;