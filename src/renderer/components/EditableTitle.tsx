import * as React from 'react';

const EditableTitle = (props: IEditableTitleProps) => {
    const [edit, setEdit] = React.useState<boolean>(false);

    const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const form = e.target as HTMLFormElement;
        const target = form.children[0] as HTMLInputElement;
        props.onSubmit(target.value);
        setEdit(false);
    }

    const onBlur = (e: React.FormEvent<HTMLInputElement>) => {
        e.preventDefault();
        const target = e.target as HTMLInputElement;
        props.onSubmit(target.value);
        setEdit(false);
    }

    if (!edit)
        return (
            <h1 className="topic-name text-center full-width" onClick={() => setEdit(true)}>
                {props.title}
            </h1>
        );

    return (
        <form onSubmit={onSubmit} className="full-width">
            <input
                className="topic-name text-center full-width"
                onBlur={onBlur}
                defaultValue={props.title}
                autoFocus={true}
                type="text"
            />
        </form>
    );
}

interface IEditableTitleProps {
    title: string
    onSubmit(value: string): void
}

export default EditableTitle;