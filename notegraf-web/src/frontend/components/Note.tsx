import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import {Link, useNavigate} from "react-router-dom";
import {deleteNote} from "../api";
import {CollectionIcon, LinkIcon, PencilAltIcon, TrashIcon} from "@heroicons/react/outline";
import katex from "katex";
import * as hljs from 'highlight.js';
import * as types from "../types";

function escapeHtml(unsafe: string): string {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}


const renderer = {
    code(code: string, infoString: string | null, escaped: boolean) {
        // @ts-ignore
        const lang = (infoString || '').match(/\S*/)[0];
        if (lang !== "math") {
            return false;
        }
        return katex.renderToString(escaped ? code : escapeHtml(code), {output: "html", displayMode: true});
    },
    codespan(code: string) {
        const match = code.match(/^\$\{(.*)}\$$/);
        if (!match) {
            return false;
        }
        return katex.renderToString(match[1], {output: "html", displayMode: false});
    }
}

function highlight(code: string, lang: string) {
    const language = hljs.default.getLanguage(lang) ? lang : 'plaintext';
    return hljs.default.highlight(code, {language}).value;
}

// @ts-ignore
marked.use({renderer, highlight: highlight});

type NoteProps = {
    note: types.Note,
    showPrevNext: boolean,
    disableControl: boolean
    setError: any,
    onDelete: () => void
}

type NoteControlProps = {
    id: string,
    setError: any,
    onDelete: () => void
}

function NoteControls(props: NoteControlProps) {
    const navigate = useNavigate();

    const onEdit = () => {
        navigate(`/note/${props.id}/edit`);
    }

    const onDelete = async () => {
        try {
            await deleteNote(props.id);
            props.onDelete();
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

export function Note(props: NoteProps) {
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
            {props.disableControl || <NoteControls id={props.note.id} setError={props.setError} onDelete={props.onDelete}/>}
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
                {/* remove the backticks with these classes prose-code:before:content-none prose-code:after:content-none */}
                <article
                    className={"overflow-hidden prose dark:prose-invert prose-github md:prose-lg lg:prose-xl xl:prose-2xl"}
                    dangerouslySetInnerHTML={{
                        __html: sanitize(marked(props.note.note_inner))
                    }}/>
            </div>
        </div>
    );
}
