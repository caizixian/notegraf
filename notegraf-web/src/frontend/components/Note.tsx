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
    PencilAltIcon,
    ReplyIcon,
    ShareIcon,
    TrashIcon
} from "@heroicons/react/outline";
import katex from "katex";
import * as hljs from 'highlight.js';
import * as types from "../types";
import {LazyLinks} from "./LazyLinks";
import {renderTitle, showAgo} from "../utils";

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
    },
    link(href: string, title: string, text: string) {
        // The original implementation cleans the URL if marked option sanitize/base
        // But these two options aren't used by Notegraf
        // The rest of the original link implementation is copied verbatim with the notegraf protocol stripping
        if (href === null) {
            return text;
        }
        if (href.indexOf("notegraf:") === 0) {
            href = href.slice(9);
        }
        let out = '<a href="' + escapeHtml(href) + '"';
        if (title) {
            out += ' title="' + title + '"';
        }
        out += '>' + text + '</a>';
        return out;
    }
}

function highlight(code: string, lang: string) {
    const language = hljs.default.getLanguage(lang) ? lang : 'plaintext';
    return hljs.default.highlight(code, {language}).value;
}

// @ts-ignore
marked.use({renderer, highlight: highlight});

type RenderMarkdownProps = {
    note_inner: string
}

export function RenderMarkdown(props: RenderMarkdownProps) {
    // remove the backticks with these classes prose-code:before:content-none prose-code:after:content-none
    return (<article
            className={"overflow-hidden prose dark:prose-invert prose-github md:prose-lg lg:prose-xl xl:prose-2xl"}
            dangerouslySetInnerHTML={{
                __html: sanitize(marked(props.note_inner))
            }}/>
    );
}

type NoteProps = {
    note: types.Note,
    showPrevNext: boolean,
    showingRevision: boolean
    setError: any,
    onDelete: () => void,
    permaLink: boolean
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
        <div className={"flex flex-wrap gap-1 my-1"}>
            <Link to={`/note/${props.note.id}/revision`}>
                <button className={"ng-button ng-button-primary"} title={"Show revisions"}>
                    <ClockIcon className={"h-6 w-6"}/>
                </button>
            </Link>
            <div className={"w-1"}></div>
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
            <div className={"w-1"}></div>
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
            <div className={"w-1"}></div>
            <button onClick={onEdit} className={"ng-button ng-button-primary"} title={"Edit"}>
                <PencilAltIcon className={"h-6 w-6"}/>
            </button>
            <button onClick={onDelete} className={"ng-button ng-button-danger ml-auto"} title={"Delete"}>
                <TrashIcon className={"h-6 w-6"}/>
            </button>
        </div>
    )
}

export function Note(props: NoteProps) {
    let link = props.permaLink ? `/note/${props.note.id}/revision/${props.note.revision}` : `/note/${props.note.id}`;
    return (
        <div className="note border border-neutral-500 my-0.5 p-1">
            <div className={"flex items-baseline"}>
                <a href={`notegraf:/note/${props.note.id}`}><LinkIcon className={"h-6 w-6"}/></a>
                <Link to={link}>
                    <h1 className={"text-4xl underline"}>{renderTitle(props.note.title)}</h1>
                </Link>
            </div>
            {props.showingRevision ||
                <NoteControls note={props.note} setError={props.setError} onDelete={props.onDelete}
                              showPrevNext={props.showPrevNext}/>}
            {props.showingRevision || <LazyLinks collectionName={"Backlinks"} noteIDs={props.note.references}/>}
            {props.showingRevision || <LazyLinks collectionName={"Branches"} noteIDs={props.note.branches}/>}
            <details className={"border-b border-neutral-500"}>
                <summary className={"select-none"}>Metadata</summary>
                <p title={props.note.metadata.created_at}>Created {showAgo(props.note.metadata.created_at)}</p>
                <p title={props.note.metadata.modified_at}>Modified {showAgo(props.note.metadata.modified_at)}</p>
                <p>Tags: {props.note.metadata.tags.join(", ")}</p>
                <p>Custom metadata: {JSON.stringify(props.note.metadata.custom_metadata)}</p>
            </details>
            <div className={"flex justify-center"}>
                <RenderMarkdown note_inner={props.note.note_inner}/>
            </div>
        </div>
    );
}
