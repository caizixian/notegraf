import * as React from "react";
import {useEffect, useState} from "react";
import {useParams} from "react-router-dom";
import {getNoteRevisions} from "../api";
import {tileInTitle} from "../utils";
import * as types from "../types";
import {NotesTwoPane} from "../components/NotesTwoPane";

export function NoteRevisions() {
    let {noteID} = useParams();
    const [notes, setNotes] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNoteRevisions() {
            try {
                const notes = await getNoteRevisions(noteID as string);
                setNotes(notes);
                setIsLoaded(true);
                document.title = `${tileInTitle(notes[notes.length - 1].title)} (revisions) - Notegraf`;
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

    return (<NotesTwoPane
        setError={setError}
        notes={notes}
        onDelete={() => {
        }}
        showAgoKey={(n: types.Note) => new Date(n.metadata.modified_at)}
        showPrevNext={false}
        showingRevision={true}
        permaLink={true}
    />);
}