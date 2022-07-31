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

export function openLinkClosure(url: string, sameTab: boolean, navigate: any) {
    return () => {
        if (sameTab) {
            navigate(url);
        } else {
            const newWindow = window.open(url, '_blank', 'noreferrer,noopener');
            if (newWindow) {
                newWindow.focus();
            }
        }
    }
}