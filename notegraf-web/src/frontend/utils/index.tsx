import * as React from "react";

export * from "./autosave";
export * from "./datetime";

export function renderTitle(title: string) {
    let className = title ? "" : "italic text-gray-500";
    return (<span className={className}>{title ? title : "no title"}</span>);
}

export function tileInTitle(title: string) {
    return title ? title : "(no title)";
}

export function openInNewTabClosure(url: string) {
    return () => {
        window.open(url, '_blank')!.focus();
    }
}