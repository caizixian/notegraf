import * as React from "react";
import {useEffect, useState} from "react";
import {useParams} from "react-router-dom";
import {getNoteRevisions} from "../api";
import {renderTitle, showAgo} from "../utils";
import {Note} from "../components/Note";
import * as types from "../types";

export function NoteRevisions() {
    let {noteID} = useParams();
    const [notes, setNotes] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);
    const [revisionSelected, setRevisionSelected] = useState<any>(null);

    useEffect(() => {
        async function fetchNoteRevisions() {
            try {
                const notes = await getNoteRevisions(noteID as string);
                setNotes(notes);
                setRevisionSelected(notes[0].revision);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNoteRevisions();
    }, [noteID]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2 flex">
        <div className={"basis-1/3 sm:basis-1/4 md:basis-1/5 lg:basis-1/6 min-w-0 divide-y divide-neutral-500"}>
            {notes.map((note: types.Note) => (<div
                key={note.revision}
                onClick={() => setRevisionSelected(note.revision)}
                className={"" + (note.revision === revisionSelected ? " bg-sky-300 dark:bg-sky-700" : "")}>
                <p className={"truncate"}>{renderTitle(note.title)}
                </p>
                <p className={"truncate"}>
                    {showAgo(note.metadata.modified_at)}
                </p>
            </div>))}
        </div>
        <div className={"ml-1 overflow-hidden basis-2/3 sm:basis-3/4 md:basis-4/5 lg:basis-5/6"}>
            <Note note={notes.find((note: types.Note) => note.revision === revisionSelected)} showPrevNext={false}
                  disableControl={true} setError={setError} onDelete={() => {
            }}/>
        </div>
    </div>);
}