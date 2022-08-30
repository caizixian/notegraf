import * as React from "react";
import {useEffect, useState} from "react";
import {renderTitle} from "../utils";
import {getNote} from "../api";
import {Tags} from "./Tags";

type LazyLinksProps = {
    collectionName: string
    noteIDs: string[]
}

export function LazyLinks(props: LazyLinksProps) {
    const [everClicked, setEverClicked] = useState(false);
    const [notes, setNotes] = useState<any[][]>([]);
    const [isLoaded, setIsLoaded] = useState(false);

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
                    {/* Keep the title and the transitive suffix on the same line, truncate title if needed */}
                    <a className={"max-w-full"}
                       href={`/note/${note[0].id}?recursiveLoad=false`}>
                        <div className={"flex gap-1"}>
                            <p className={"truncate underline"}>{renderTitle(note[1].title)}</p>
                            {note[2] && (<p className={"italic text-gray-500"}>(transitive)</p>)}
                        </div>
                    </a>
                    {/* If tags don't fit on the current line, wrap them to the next line */}
                    <Tags tags={note[0].metadata.tags} disableLink={false}></Tags>
                </div>
            </li>);
    } else {
        listItems = props.noteIDs.map(noteID =>
            <li key={noteID}>
                <a className={"min-w-0 truncate underline"}
                   href={`/note/${noteID}`}>
                    {noteID}
                </a>
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