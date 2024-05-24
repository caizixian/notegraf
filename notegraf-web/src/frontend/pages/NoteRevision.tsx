import * as React from "react";
import {useEffect, useState} from "react";
import {useParams} from "react-router-dom";
import {getNoteSpecific} from "../api";
import {Note} from "../components/Note";
import {showAgo, tileInTitle} from "../utils";

export function NoteRevision() {
    let {noteID, revision} = useParams();
    const [note, setNote] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNoteSepcific() {
            try {
                const note = await getNoteSpecific(noteID as string, revision as string);
                setNote(note);
                setIsLoaded(true);
                document.title = `${tileInTitle(note.title)} (${showAgo(new Date(note.metadata.modified_at))}) - Notegraf`;
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNoteSepcific();
    }, [noteID, revision]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2 min-h-0 overflow-y-auto">
        <Note note={note} key={note.id} showPrevNext={false} permaLink={true}
              setError={setError} showingRevision={true} onDelete={() => {
        }}></Note>
    </div>);
}