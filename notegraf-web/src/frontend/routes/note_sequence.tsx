import * as React from "react";
import {useEffect, useState} from "react";
import {useParams, useSearchParams} from "react-router-dom";
import {Note, NoteComponent} from "../note"
import {getNote} from "../api";

async function fetchNoteSequence(anchorNoteID: string, recursiveLoad: boolean): Promise<Note[]> {
    let notes: Note[] = [];
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
    let {anchorNoteID} = useParams();
    let [searchParams, setSearchParams] = useSearchParams();
    const [notes, setNotes] = useState<Note[]>([]);
    const [error, setError] = useState<any>(null);
    const recursiveLoad = searchParams.get("recursiveLoad") === "true";

    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNoteSequenceInner() {
            try {
                const notes = await fetchNoteSequence(anchorNoteID as string, recursiveLoad);
                setNotes(notes);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNoteSequenceInner();
    }, [anchorNoteID, recursiveLoad]);

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
            <label htmlFor={"recursiveLoad"}>Recursive load?</label>
        </div>
        {notes.map(note => (<NoteComponent note={note} key={note.id} showPrevNext={!recursiveLoad}
                                           setError={setError}></NoteComponent>))}
    </div>);
}