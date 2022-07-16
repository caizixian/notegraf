import * as React from "react";
import {useEffect, useState} from "react";
import {useParams} from "react-router-dom";
import {getNoteSpecific} from "../api";
import {Note} from "../components/Note";

export function NoteRevision() {
    let {anchorNoteID, revision} = useParams();
    const [note, setNote] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNoteSepcific() {
            try {
                const note = await getNoteSpecific(anchorNoteID as string, revision as string);
                setNote(note);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNoteSepcific();
    }, [anchorNoteID, revision]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2">
        <Note note={note} key={note.id} showPrevNext={false}
              setError={setError} disableControl={true} onDelete={() => {
        }}></Note>
    </div>);
}