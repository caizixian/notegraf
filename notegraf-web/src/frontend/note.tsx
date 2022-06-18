import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import "./app.css"

export type Note = {
    note_inner: string,
    id: string,
    revision: string,
    parent: string | null,
    branches: string[],
    prev: string | null,
    next: string | null,
    references: string[],
    referents: string[],
    metadata: NoteMetadata
}

type NoteMetadata = {
    schema_version: number,
    created_at: string,
    modified_at: string,
    tags: string[],
    custom_metadata: any
}

type NoteComponentProps = {
    note: Note,
    showPrevNext: boolean
}

type Empty = Record<any, never>;

export class NoteComponent extends React.Component<NoteComponentProps, Empty> {
    constructor(props: NoteComponentProps) {
        super(props);
    }

    render() {
        return (
            <article className="note border m-1 p-1">
                {this.props.showPrevNext &&
                    <div>
                        {this.props.note.prev != null && <a href={"/note/" + this.props.note.prev} className={"underline text-blue-500 m-0.5"}>prev</a>}
                        {this.props.note.next != null && <a href={"/note/" + this.props.note.next} className={"underline text-blue-500 m-0.5"}>next</a>}
                    </div>}
                <details>
                    <summary>Metadata</summary>
                    <p>Created at: {this.props.note.metadata.created_at}</p>
                    <p>Modified at: {this.props.note.metadata.modified_at}</p>
                </details>
                <div dangerouslySetInnerHTML={{
                    __html: sanitize(marked(this.props.note.note_inner))
                }}/>
            </article>
        );
    }
}