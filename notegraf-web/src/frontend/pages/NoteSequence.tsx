import * as React from "react";
import {useEffect, useState} from "react";
import {useParams, useSearchParams} from "react-router-dom";
import {getNote} from "../api";
import {Note} from "../components/Note";
import * as types from "../types";
import {tileInTitle} from "../utils";

async function fetchNoteSequence(anchorNoteID: string, recursiveLoad: boolean): Promise<types.Note[]> {
    let notes: types.Note[] = [];
    let anchorNote = await getNote(anchorNoteID);
    notes.push(anchorNote);
    if (recursiveLoad) {
        while (notes[0].prev != null) {
            let note = await getNote(notes[0].prev);
            notes = [note, ...notes];
        }
        while (notes[notes.length - 1].next != null) {
            let note = await getNote(notes[notes.length - 1].next as string);
            notes.push(note);
        }
    }
    return notes;
}

export function NoteSequence() {
    let {noteID} = useParams();
    let [searchParams, setSearchParams] = useSearchParams();
    const [notes, setNotes] = useState<types.Note[]>([]);
    const [error, setError] = useState<any>(null);
    const recursiveLoad = searchParams.get("recursiveLoad") === "true";

    const [isLoaded, setIsLoaded] = useState(false);

    async function fetchNoteSequenceInner() {
        try {
            const notes = await fetchNoteSequence(noteID as string, recursiveLoad);
            setNotes(notes);
            setIsLoaded(true);
            document.title = `${tileInTitle(notes[0].title)}${recursiveLoad ? " (recursive)" : ""} - Notegraf`;
        } catch (e) {
            setError(e);
            setIsLoaded(true);
        }
    }

    useEffect(() => {
        fetchNoteSequenceInner();
    }, [noteID, recursiveLoad]);

    function handleCheckbox(event: React.FormEvent<HTMLInputElement>) {
        const checked = event.currentTarget.checked;
        setSearchParams({
            recursiveLoad: checked.toString()
        });
    }

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2">
        <div>
            <input type="checkbox" id="recursiveLoad" name="recursiveLoad" checked={recursiveLoad}
                   onChange={handleCheckbox}/>
            <label htmlFor={"recursiveLoad"} className={"select-none"}>Recursive load?</label>
        </div>
        {notes.map(note => (<Note note={note} key={note.id} showPrevNext={!recursiveLoad} permaLink={false}
                                  setError={setError} disableControl={false}
                                  onDelete={fetchNoteSequenceInner}></Note>))}
    </div>);
}