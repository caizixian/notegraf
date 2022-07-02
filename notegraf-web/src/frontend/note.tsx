import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import "./app.css"
import {Link} from "react-router-dom";

export type Note = {
    title: string,
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
        <article className="note border my-0.5 px-1">
            <Link to={`/note/${props.note.id}/`} className={"underline decoration-blue-500"}>
                {props.note.title ? <h1>{props.note.title}</h1> : <h1 className={"italic text-gray-500"}>no title</h1>}
            </Link>
            {props.showPrevNext &&
                <div>
                    {props.note.prev != null && <Link to={`../${props.note.prev}`} key={props.note.prev}
                                                      className={"underline text-blue-500 m-0.5"}>prev</Link>}
                    {props.note.next != null && <Link to={`../${props.note.next}`} key={props.note.next}
                                                      className={"underline text-blue-500 m-0.5"}>next</Link>}
                </div>}
            <details className={"border-b border-gray-500"}>
                <summary>Metadata</summary>
                <p>Created at: {props.note.metadata.created_at}</p>
                <p>Modified at: {props.note.metadata.modified_at}</p>
                <p>Tags: {props.note.metadata.tags.join(", ")}</p>
                <p>Custom metadata: {JSON.stringify(props.note.metadata.custom_metadata)}</p>
                <Link to={`/note/${props.note.id}/revision/${props.note.revision}`}
                      className={"underline text-blue-500"}>
                    Permalink
                </Link>
            </details>
            <div dangerouslySetInnerHTML={{
                __html: sanitize(marked(props.note.note_inner))
            }}/>
        </article>
    );
}
