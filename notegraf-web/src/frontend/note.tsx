import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import {Link, useNavigate} from "react-router-dom";
import {deleteNote} from "./api";
import {CollectionIcon, LinkIcon, PencilAltIcon, TrashIcon} from "@heroicons/react/outline";

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
    showPrevNext: boolean,
    disableControl: boolean
    setError: any
}

type NoteControlProps = {
    id: string
    setError: any
}

function NoteControls(props: NoteControlProps) {
    const navigate = useNavigate();

    const onEdit = () => {
        navigate(`/note/${props.id}/edit`);
    }

    const onDelete = async () => {
        try {
            await deleteNote(props.id);
            navigate("/note");
        } catch (e) {
            props.setError(e);
        }
    }

    return (
        <div className={"flex gap-1 my-1"}>
            <Link to={`/note/${props.id}/revision`}>
                <button className={"ng-button ng-button-primary"}>
                    <CollectionIcon className={"h-6 w-6"}/>
                </button>
            </Link>
            <button onClick={onEdit} className={"ng-button ng-button-primary"}>
                <PencilAltIcon className={"h-6 w-6"}/>
            </button>
            <button onClick={onDelete} className={"ng-button ng-button-danger"}>
                <TrashIcon className={"h-6 w-6"}/>
            </button>
        </div>
    )
}

export function NoteComponent(props: NoteComponentProps) {
    return (
        <div className="note border border-neutral-500 my-0.5 p-1">
            <div className={"flex items-baseline"}>
                <a href={`notegraf:/note/${props.note.id}`}><LinkIcon className={"h-6 w-6"}/></a>
                <h1 className={"text-4xl"}>{props.note.title ? props.note.title :
                    <span className={"italic text-gray-500"}>no title</span>}</h1>
            </div>
            {props.showPrevNext &&
                <div>
                    {props.note.prev != null && <Link to={`../${props.note.prev}`} key={props.note.prev}
                                                      className={"underline text-blue-500 m-0.5"}>prev</Link>}
                    {props.note.next != null && <Link to={`../${props.note.next}`} key={props.note.next}
                                                      className={"underline text-blue-500 m-0.5"}>next</Link>}
                </div>}
            {props.disableControl || <NoteControls id={props.note.id} setError={props.setError}/>}
            <details className={"border-b border-neutral-500"}>
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
            <div className={"flex justify-center"}>
                <article
                    className={"overflow-hidden prose dark:prose-invert md:prose-lg lg:prose-xl xl:prose-2xl prose-code:before:content-none prose-code:after:content-none"}
                    dangerouslySetInnerHTML={{
                        __html: sanitize(marked(props.note.note_inner))
                    }}/>
            </div>
        </div>
    );
}
