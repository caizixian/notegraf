import {marked} from "marked";
import katex from "katex";
import * as hljs from 'highlight.js';

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