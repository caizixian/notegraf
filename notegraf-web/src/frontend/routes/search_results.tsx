import * as React from "react";
import {useEffect, useState} from "react";
import {useSearchParams} from "react-router-dom";
import {Note, NoteComponent} from "../note"
import {searchNotes} from "../api";
import {showAgo} from "../utils/datetime";

export function SearchResults() {
    let [searchParams, _setSearchParams] = useSearchParams();
    let query = searchParams.get("query");

    const [notes, setNotes] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);
    const [revisionSelected, setRevisionSelected] = useState<any>(null);

    useEffect(() => {
        async function fetchSearch() {
            try {
                const notes: Note[] = await searchNotes(query ? query : "");
                if (notes.length === 0) {
                    setError("No match!");
                    setIsLoaded(true);
                    return;
                }
                setError(null);
                setNotes(notes);
                setRevisionSelected(notes[0].revision);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchSearch();
    }, [searchParams]);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<div className="p-2 flex">
        <div className={"basis-1/3 sm:basis-1/4 md:basis-1/5 lg:basis-1/6 min-w-0 divide-y divide-neutral-500"}>
            {notes.map((note: Note) => (<div
                key={note.id}
                onClick={() => setRevisionSelected(note.revision)}
                className={"" + (note.revision === revisionSelected ? " bg-sky-300 dark:bg-sky-700" : "")}>
                <p className={"truncate"}>{note.title ? note.title : <span
                    className={"italic text-gray-500"}>no title</span>}
                </p>
                <p className={"truncate"}>
                    {showAgo(note.metadata.created_at)}
                </p>
            </div>))}
        </div>
        <div className={"ml-1 overflow-hidden basis-2/3 sm:basis-3/4 md:basis-4/5 lg:basis-5/6"}>
            <NoteComponent note={notes.find((note: Note) => note.revision === revisionSelected)} showPrevNext={false}
                           disableControl={false} setError={setError}/>
        </div>
    </div>);
}
