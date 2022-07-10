import * as React from "react";
import {useEffect, useState} from "react";
import {useParams} from "react-router-dom";
import {Note, NoteComponent} from "../note"
import {getNoteRevisions} from "../api";

export function NoteRevisions() {
    let {anchorNoteID} = useParams();
    const [notes, setNotes] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNoteRevisions() {
            try {
                const notes: Note[] = await getNoteRevisions(anchorNoteID as string);
                setNotes(notes);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNoteRevisions();
    }, [anchorNoteID]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2">
        {notes.map((note: Note) => (<NoteComponent note={note} key={note.id} showPrevNext={false}
                                                   setError={setError} disableControl={true}></NoteComponent>))}
    </div>);
}