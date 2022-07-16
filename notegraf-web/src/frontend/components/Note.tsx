import * as React from "react";
import {marked} from "marked";
import {sanitize} from "dompurify";
import {Link, useNavigate} from "react-router-dom";
import {deleteNote} from "../api";
import {
    ArrowDownIcon,
    ArrowUpIcon,
    ClockIcon,
    CollectionIcon,
    LinkIcon,
    PencilAltIcon, ReplyIcon,
    ShareIcon,
    TrashIcon
} from "@heroicons/react/outline";
import katex from "katex";
import * as hljs from 'highlight.js';
import * as types from "../types";
import {LazyLinks} from "./LazyLinks";
import {renderTitle} from "../utils";

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
    note: types.Note,
    setError: any,
    onDelete: () => void,
    showPrevNext: boolean
}

function NoteControls(props: NoteControlProps) {
    const navigate = useNavigate();

    const onEdit = () => {
        navigate(`/note/${props.note.id}/edit`);
    }

    const onDelete = async () => {
        try {
            await deleteNote(props.note.id);
            props.onDelete();
        } catch (e) {
            props.setError(e);
        }
    }

    return (
        <div className={"flex gap-1 my-1"}>
            <Link to={`/note/${props.note.id}/revision`}>
                <button className={"ng-button ng-button-primary"} title={"Show revisions"}>
                    <ClockIcon className={"h-6 w-6"}/>
                </button>
            </Link>
            {props.note.parent != null && <Link to={`/note/${props.note.parent}`}>
                <button className={"ng-button ng-button-primary"} title={"Parent"}>
                    <ReplyIcon className={"h-6 w-6"}/>
                </button>
            </Link>}
            {props.showPrevNext && props.note.prev != null && <Link to={`/note/${props.note.prev}`}>
                <button className={"ng-button ng-button-primary"} title={"Previous"}>
                    <ArrowUpIcon className={"h-6 w-6"}/>
                </button>
            </Link>}
            {props.showPrevNext && props.note.next != null && <Link to={`/note/${props.note.next}`}>
                <button className={"ng-button ng-button-primary"} title={"Next"}>
                    <ArrowDownIcon className={"h-6 w-6"}/>
                </button>
            </Link>}
            <Link to={`/note/${props.note.id}/branch`}>
                <button className={"ng-button ng-button-primary"} title={"Add branch"}>
                    <ShareIcon className={"h-6 w-6"}/>
                </button>
            </Link>
            {props.note.next != null || <Link to={`/note/${props.note.id}/append`}>
                <button className={"ng-button ng-button-primary"} title={"Append note"}>
                    <CollectionIcon className={"h-6 w-6"}/>
                </button>
            </Link>}
            <button onClick={onEdit} className={"ng-button ng-button-primary"} title={"Edit"}>
                <PencilAltIcon className={"h-6 w-6"}/>
            </button>
            <button onClick={onDelete} className={"ng-button ng-button-danger"} title={"Delete"}>
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
                <Link to={`/note/${props.note.id}`}>
                    <h1 className={"text-4xl underline"}>{renderTitle(props.note.title)}</h1>
                </Link>
            </div>
            {props.disableControl ||
                <NoteControls note={props.note} setError={props.setError} onDelete={props.onDelete}
                              showPrevNext={props.showPrevNext}/>}
            <LazyLinks collectionName={"Backlinks"} noteIDs={props.note.references}/>
            <LazyLinks collectionName={"Branches"} noteIDs={props.note.branches}/>
            <details className={"border-b border-neutral-500"}>
                <summary className={"select-none"}>Metadata</summary>
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
