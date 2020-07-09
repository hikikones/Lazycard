import * as React from 'react';

const Empty = (props: IEmptyProps) => {
    return (
        <div className="col col-center row-center full-height">
            <section className="col col-center empty-state text-center">
                <i className="material-icons icon">{props.icon}</i>
                <h3>{props.message}</h3>
            </section>
            {props.children &&
                <section>
                    {props.children}
                </section>
            }
        </div>
    );
}

interface IEmptyProps {
    icon: string
    message: string
    children?: React.ReactNode
}

export default Empty;