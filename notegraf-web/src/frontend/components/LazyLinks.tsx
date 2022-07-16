import * as React from "react";
import {useEffect, useState} from "react";
import {Link} from "react-router-dom";
import * as types from "../types";
import {renderTitle} from "../utils";
import {getNote} from "../api";

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
            setNotes(notes);
            setIsLoaded(true);
        }
    }

    useEffect(() => {
        loadNotes(props.noteIDs);
    }, [everClicked]);
    let listItems;
    if (isLoaded) {
        listItems = notes.map(note =>
            <li key={note.id}>
                <Link to={`/note/${note.id}`} className={"underline"}>
                    <p className={"truncate"}>{renderTitle(note.title)}</p>
                </Link>
            </li>);
    } else {
        listItems = props.noteIDs.map(noteID =>
            <li key={noteID}>
                <Link to={`/note/${noteID}`} className={"underline"}>
                    <p className={"truncate"}>{noteID}</p>
                </Link>
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