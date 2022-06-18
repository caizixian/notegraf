import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import "./app.css"
import {Link} from "react-router-dom";

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

export function NoteComponent(props: NoteComponentProps) {
    return (
        <article className="note border m-1 p-1">
            {props.showPrevNext &&
                <div>
                    {props.note.prev != null && <Link to={`../${props.note.prev}`} key={props.note.prev}
                                                      className={"underline text-blue-500 m-0.5"}>prev</Link>}
                    {props.note.next != null && <Link to={`../${props.note.next}`} key={props.note.next}
                                                      className={"underline text-blue-500 m-0.5"}>next</Link>}
                </div>}
            <details>
                <summary>Metadata</summary>
                <p>Created at: {props.note.metadata.created_at}</p>
                <p>Modified at: {props.note.metadata.modified_at}</p>
            </details>
            <div dangerouslySetInnerHTML={{
                __html: sanitize(marked(props.note.note_inner))
            }}/>
        </article>
    );
}
