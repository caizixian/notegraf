import * as React from "react";
import {useState} from "react";
import {marked, RendererObject, Tokens} from "marked";
import {markedHighlight} from "marked-highlight";
import {markedSmartypants} from "marked-smartypants";
import {gfmHeadingId} from "marked-gfm-heading-id";
import {sanitize} from "dompurify";
import {Link} from "react-router-dom";
import {deleteNote} from "../api";
import {
    ArrowDownIcon,
    ArrowUpIcon,
    ArrowUturnLeftIcon,
    ClockIcon,
    CodeBracketIcon,
    DocumentTextIcon,
    LinkIcon,
    PencilSquareIcon,
    RectangleStackIcon,
    ShareIcon,
    TrashIcon
} from "@heroicons/react/24/outline";
import * as katex from "katex";
import * as hljs from 'highlight.js';
import * as types from "../types";
import {LazyLinks} from "./LazyLinks";
import {renderTitle, showAgo} from "../utils";
import {Tags} from "./Tags";

function escapeHtml(unsafe: string): string {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

function cleanUrl(href: string) {
    try {
        href = encodeURI(href).replace(/%25/g, '%');
    } catch {
        return null;
    }
    return href;
}

function renderMath(escaped: string, displayMode: boolean): string | false {
    const doc = new DOMParser().parseFromString(escaped, "text/html");
    const parsed = doc.documentElement.textContent!;
    try {
        return katex.renderToString(parsed, {output: "html", displayMode: displayMode});
    } catch (e) {
        if (e instanceof katex.ParseError) {
            if (displayMode) {
                return `<pre><code class="text-red-500">${escapeHtml(e.toString())}</code><br/>` +
                    `<code>${escapeHtml(parsed)}</code></pre>`;
            } else {
                return `<code title="${escapeHtml(e.toString())}" class="text-red-500">${escapeHtml(parsed)}</code>`;
            }
        } else {
            return false;
        }
    }
}

const renderer: RendererObject = {
    code({text, lang, escaped}: Tokens.Code) {
        const langString = (lang || '').match(/^\S*/)?.[0];
        if (langString !== "math") {
            return false;
        }
        return renderMath(escaped ? text : escapeHtml(text), true);
    },
    codespan({text}: Tokens.Codespan) {
        const match = text.match(/^\$\{(.*)}\$$/);
        if (!match) {
            return false;
        }
        return renderMath(match[1], false);
    },
    link({href, title, tokens}: Tokens.Link) {
        const text = this.parser.parseInline(tokens);
        const cleanHref = cleanUrl(href);
        if (cleanHref === null) {
            return text;
        }
        href = cleanHref;
        if (href.indexOf("notegraf:") === 0) {
            href = href.slice(9);
        }
        const isExternalURL = new URL(href, location.origin).origin !== location.origin;
        let out = '<a href="' + href + '"';
        if (title) {
            out += ' title="' + title + '"';
        }
        out += '>';
        out += text;
        if (isExternalURL) {
            out += '<svg xmlns="http://www.w3.org/2000/svg" class="h-[1em] w-[1em] inline" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">' +
                '<path stroke-linecap="round" stroke-linejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />' +
                '</svg>';
        }
        out += '</a>';
        return out;
    },
    // Workaround for https://github.com/markedjs/marked/issues/1486
    listitem(item: Tokens.ListItem) {
        let itemBody = '';
        if (item.task) {
            itemBody += "<label>"
            const checkbox = this.checkbox({checked: !!item.checked});
            if (item.loose) {
                if (item.tokens.length > 0 && item.tokens[0].type === 'paragraph') {
                    item.tokens[0].text = checkbox + ' ' + item.tokens[0].text;
                    if (item.tokens[0].tokens && item.tokens[0].tokens.length > 0 && item.tokens[0].tokens[0].type === 'text') {
                        item.tokens[0].tokens[0].text = checkbox + ' ' + item.tokens[0].tokens[0].text;
                    }
                } else {
                    item.tokens.unshift({
                        type: 'text',
                        raw: checkbox + ' ',
                        text: checkbox + ' ',
                    });
                }
            } else {
                itemBody += checkbox + ' ';
            }
        }

        itemBody += this.parser.parse(item.tokens, !!item.loose);

        if (item.task) {
            itemBody += "</label>";
        }
        return `<li>${itemBody}</li>\n`;
    }
}

function highlight(code: string, lang: string) {
    const language = hljs.default.getLanguage(lang) ? lang : 'plaintext';
    return hljs.default.highlight(code, {language}).value;
}

// @ts-ignore
marked.use(markedSmartypants(), markedHighlight({highlight}), gfmHeadingId(), {renderer});

type RenderMarkdownProps = {
    note_inner: string,
    rendered: boolean
}

export function RenderMarkdown(props: RenderMarkdownProps) {
    // TODO: remove the backticks with these classes prose-code:before:content-none prose-code:after:content-none
    // Add horizontal scrolling for words that are too long and cannot be broken
    const proseClasses: string = "overflow-x-auto prose dark:prose-invert prose-github md:prose-lg lg:prose-xl xl:prose-2xl";
    if (props.rendered) {
        return (<article
            className={proseClasses}
            dangerouslySetInnerHTML={{
                __html: sanitize(marked(props.note_inner) as string)
            }}/>);
    } else {
        return (
            <article className={proseClasses}>
                <pre>
                    <code className={"language-markdown"}
                          dangerouslySetInnerHTML={{
                              __html: sanitize(highlight(props.note_inner, "markdown"))
                          }}>
                    </code>
                </pre>
            </article>
        );
    }
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
    showPrevNext: boolean,
    rendered: boolean
    toggleRendered: () => void
}

function NoteControls(props: NoteControlProps) {
    const onDelete = async () => {
        try {
            if (window.confirm("Are you sure you want to delete this note?")) {
                await deleteNote(props.note.id);
                props.onDelete();
            }
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
                    <ArrowUturnLeftIcon className={"h-6 w-6"}/>
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
                    <RectangleStackIcon className={"h-6 w-6"}/>
                </button>
            </Link>}
            <div className={"w-1"}></div>
            {props.rendered ?
                <button className={"ng-button ng-button-primary"} title={"Show source"} onClick={(e) => {
                    e.preventDefault();
                    props.toggleRendered();
                }}>
                    <CodeBracketIcon className={"h-6 w-6"}/>
                </button> :
                <button className={"ng-button ng-button-primary"} title={"Show rendered"} onClick={(e) => {
                    e.preventDefault();
                    props.toggleRendered();
                }}>
                    <DocumentTextIcon className={"h-6 w-6"}/>
                </button>
            }
            <Link to={`/note/${props.note.id}/edit`}>
                <button className={"ng-button ng-button-primary"} title={"Edit"}>
                    <PencilSquareIcon className={"h-6 w-6"}/>
                </button>
            </Link>
            <button onClick={onDelete} className={"ng-button ng-button-danger ml-auto"} title={"Delete"}>
                <TrashIcon className={"h-6 w-6"}/>
            </button>
        </div>
    )
}

export function Note(props: NoteProps) {
    let link = props.permaLink ? `/note/${props.note.id}/revision/${props.note.revision}` : `/note/${props.note.id}`;
    const [rendered, setRendered] = useState(true); // Show rendered note or not
    const toggleRendered = () => {
        setRendered(!rendered);
    };
    return (
        <div className="note border border-neutral-500 my-0.5 p-1">
            <div className={"flex items-baseline mb-1.5"}>
                <a href={`notegraf:/note/${props.note.id}`}><LinkIcon className={"h-6 w-6"}/></a>
                {/* The below break a very long title into multiple lines. If a single word is too long, we show ... */}
                <a href={link} className={"underline min-w-0"}>
                    <h1 className={"text-4xl text-ellipsis overflow-hidden"}>
                        {renderTitle(props.note.title)}
                    </h1>
                </a>
            </div>
            <Tags tags={props.note.metadata.tags} disableLink={false}/>
            {props.showingRevision ||
                <NoteControls note={props.note} setError={props.setError} onDelete={props.onDelete}
                              showPrevNext={props.showPrevNext} rendered={rendered} toggleRendered={toggleRendered}/>}
            {props.showingRevision || <LazyLinks collectionName={"Backlinks"} noteIDs={props.note.references}/>}
            {props.showingRevision || <LazyLinks collectionName={"Branches"} noteIDs={props.note.branches}/>}
            <details className={"border-b border-neutral-500"}>
                <summary className={"select-none"}>Metadata</summary>
                <p title={props.note.metadata.created_at}>Created {showAgo(new Date(props.note.metadata.created_at))}</p>
                <p title={props.note.metadata.modified_at}>Modified {showAgo(new Date(props.note.metadata.modified_at))}</p>
                <p>Custom metadata: {JSON.stringify(props.note.metadata.custom_metadata)}</p>
            </details>
            <div className={"flex justify-center"}>
                <RenderMarkdown note_inner={props.note.note_inner} rendered={rendered}/>
            </div>
        </div>
    );
}
