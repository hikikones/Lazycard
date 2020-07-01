import * as React from 'react';

const EditableHeader = (props: IEditableHeaderProps) => {
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
            <h1 className="topic-name" onClick={() => setEdit(true)}>
                {props.title}
            </h1>
        );

    return (
        <form onSubmit={onSubmit}>
            <input
                className="topic-name"
                onBlur={onBlur}
                defaultValue={props.title}
                autoFocus={true}
                type="text"
            />
        </form>
    );
}

interface IEditableHeaderProps {
    title: string
    onSubmit(value: string): void
}

export default EditableHeader;