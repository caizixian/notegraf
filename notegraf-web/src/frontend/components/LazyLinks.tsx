import * as React from "react";
import {useEffect, useState} from "react";
import * as types from "../types";
import {openInNewTabClosure, renderTitle} from "../utils";
import {getNote} from "../api";
import {Tags} from "./Tags";

type LazyLinksProps = {
    collectionName: string
    noteIDs: string[]
}

export function LazyLinks(props: LazyLinksProps) {
    const [everClicked, setEverClicked] = useState(false);
    const [notes, setNotes] = useState<types.Note[]>([]);
    const [isLoaded, setIsLoaded] = useState(false);

    function onToggle() {
        setEverClicked(true);
    }

    async function loadNotes(noteIDs: string[]) {
        if (everClicked) {
            let notes = await Promise.all(noteIDs.map(async (noteID) => {
                return await getNote(noteID);
            }));
            notes.sort((a, b) => a.title.localeCompare(b.title));
            setNotes(notes);
            setIsLoaded(true);
        }
    }

    useEffect(() => {
        loadNotes(props.noteIDs);
    }, [everClicked, props.noteIDs]);

    let listItems;
    if (isLoaded) {
        listItems = notes.map(note =>
            <li key={note.id}>
                <div className={"flex flex-wrap gap-1"}>
                    <p className={"min-w-0 truncate underline cursor-pointer"}
                       onClick={openInNewTabClosure(`/note/${note.id}`)}>
                        {renderTitle(note.title)}
                    </p>
                    <Tags tags={note.metadata.tags} disableLink={false}></Tags>
                </div>
            </li>);
    } else {
        listItems = props.noteIDs.map(noteID =>
            <li key={noteID}>
                <p className={"truncate underline cursor-pointer"}
                   onClick={openInNewTabClosure(`/note/${noteID}`)}>
                    {noteID}
                </p>
            </li>);
    }
    return (
        <details onToggle={onToggle}>
            <summary className={"select-none"}>
                {props.collectionName} ({props.noteIDs.length})
            </summary>
            <ul>{listItems}
            </ul>
        </details>
    );
}