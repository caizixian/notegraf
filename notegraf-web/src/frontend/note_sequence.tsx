import './app.css';
import * as React from "react";
import {useEffect, useState} from "react";
import {Note, NoteComponent} from "./note"
import {useParams, useSearchParams} from "react-router-dom";

async function fetchNote(noteID: string): Promise<Note> {
    const response = await fetch(`/api/v1/note/${noteID}`);
    if (!response.ok) {
        throw new Error(response.statusText);
    }
    return response.json();
}

async function fetchNoteSequence(anchorNoteID: string, recursiveLoad: boolean): Promise<Note[]> {
    let notes: Note[] = [];
    let anchorNote = await fetchNote(anchorNoteID);
    notes.push(anchorNote);
    if (recursiveLoad) {
        while (notes[0].prev != null) {
            let note = await fetchNote(notes[0].prev);
            notes = [note, ...notes];
        }
        while (notes[notes.length - 1].next != null) {
            let note = await fetchNote(notes[notes.length - 1].next as string);
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

    return (<div className="note-sequence">
        <label>
            <input type="checkbox" id="recursiveLoad" name="recursiveLoad" checked={recursiveLoad}
                   onChange={handleCheckbox}/>
            Recursive Load
        </label>
        {notes.map(note => (<NoteComponent note={note} key={note.id} showPrevNext={!recursiveLoad}></NoteComponent>))}
    </div>);
}