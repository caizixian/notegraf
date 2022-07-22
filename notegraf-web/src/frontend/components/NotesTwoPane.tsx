import * as types from "../types";
import * as React from "react";
import {useEffect, useState} from "react";
import {renderTitle, showAgo} from "../utils";
import {Note, Tags} from "./Note";

type NotesTwoPaneProps = {
    setError: any,
    notes: types.Note[],
    onDelete: any,
    showAgoKey: any,
    showPrevNext: boolean,
    permaLink: boolean,
    showingRevision: boolean
}

export function NotesTwoPane(props: NotesTwoPaneProps) {
    const [revisionSelected, setRevisionSelected] = useState<any>(null);

    useEffect(() => {
            setRevisionSelected(props.notes[0].revision);
        }, [props.notes]
    );

    const noteSelected = props.notes.find((note: types.Note) => note.revision === revisionSelected);
    const noteToShow = noteSelected ? noteSelected : props.notes[0];

    return (<div className="p-2 flex">
        <div className={"basis-1/3 sm:basis-1/4 md:basis-1/5 lg:basis-1/6 min-w-0 divide-y divide-neutral-500"}>
            {props.notes.map((note: types.Note) => (<div
                key={note.revision}
                onClick={() => setRevisionSelected(note.revision)}
                className={"my-1" + (note.revision === revisionSelected ? " bg-sky-300 dark:bg-sky-700" : "")}>
                <p className={"truncate"}>{renderTitle(note.title)}</p>
                <p className={"truncate"}>
                    {showAgo(props.showAgoKey(note))}
                </p>
                <Tags tags={note.metadata.tags} disableLink={true}/>
            </div>))}
        </div>
        <div className={"ml-1 overflow-hidden basis-2/3 sm:basis-3/4 md:basis-4/5 lg:basis-5/6"}>
            <Note note={noteToShow}
                  showPrevNext={props.showPrevNext}
                  showingRevision={props.showingRevision}
                  setError={props.setError}
                  onDelete={props.onDelete}
                  permaLink={props.permaLink}
            />
        </div>
    </div>);
}