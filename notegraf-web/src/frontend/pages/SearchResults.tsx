import * as React from "react";
import {useEffect, useState} from "react";
import {useSearchParams} from "react-router-dom";
import {searchNotes} from "../api";
import * as types from "../types";
import {NotesTwoPane} from "../components/NotesTwoPane";

export function SearchResults() {
    let [searchParams, _setSearchParams] = useSearchParams();
    let query = searchParams.get("query");

    const [notes, setNotes] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    async function fetchSearch() {
        try {
            const notes: types.Note[] = await searchNotes(query ? query : "");
            if (notes.length === 0) {
                setError("No match!");
                setIsLoaded(true);
                return;
            }
            setError(null);
            setNotes(notes);
            setIsLoaded(true);
            if (query) {
                document.title = `${query} (search) - Notegraf`;
            } else {
                document.title = `(recent) - Notegraf`;
            }
        } catch (e) {
            setError(e);
            setIsLoaded(true);
        }
    }

    useEffect(() => {
        fetchSearch();
    }, [searchParams]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        console.log(error);
        return (<div>{error.toString()}</div>);
    }

    return (<NotesTwoPane
        setError={setError}
        notes={notes}
        onDelete={fetchSearch}
        showAgoKey={(n: types.Note) => new Date(n.metadata.created_at)}
        showPrevNext={true}
        showingRevision={false}
        permaLink={false}
    />);
}

