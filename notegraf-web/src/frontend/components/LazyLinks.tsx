import * as React from "react";
import {useEffect, useState} from "react";
import {openLinkClosure, renderTitle} from "../utils";
import {getNote} from "../api";
import {Tags} from "./Tags";
import {useNavigate} from "react-router-dom";

type LazyLinksProps = {
    collectionName: string
    noteIDs: string[]
}

export function LazyLinks(props: LazyLinksProps) {
    const [everClicked, setEverClicked] = useState(false);
    const [notes, setNotes] = useState<any[][]>([]);
    const [isLoaded, setIsLoaded] = useState(false);
    const navigate = useNavigate();

    function onToggle() {
        setEverClicked(true);
    }

    async function loadNote(noteID: string): Promise<any[]> {
        const baseNote = await getNote(noteID);
        let rootNote = baseNote;
        let transitive = false;
        while (rootNote.title == "" && rootNote.prev != null) {
            rootNote = await getNote(rootNote.prev);
        }

        if (rootNote != baseNote) {
            transitive = true;
        }
        return [baseNote, rootNote, transitive];
    }

    async function loadNotes(noteIDs: string[]) {
        if (everClicked) {
            let notes = await Promise.all(noteIDs.map(async (noteID) => {
                return await loadNote(noteID);
            }));
            notes.sort((a, b) => a[1].title.localeCompare(b[1].title));
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
            <li key={note[0].id}>
                <div className={"flex flex-wrap gap-1"}>
                    <p className={"min-w-0 truncate underline cursor-pointer"}
                       onClick={openLinkClosure(`/note/${note[0].id}?recursiveLoad=false`, true, navigate)}>
                        {renderTitle(note[1].title)}
                    </p>
                    {note[2] && (<span className={"italic text-gray-500"}> (transitive)</span>)}
                    <Tags tags={note[0].metadata.tags} disableLink={false}></Tags>
                </div>
            </li>);
    } else {
        listItems = props.noteIDs.map(noteID =>
            <li key={noteID}>
                <p className={"truncate underline cursor-pointer"}
                   onClick={openLinkClosure(`/note/${noteID}`, true, navigate)}>
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