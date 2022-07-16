import * as React from "react";
import {NoteForm} from "../components/NoteForm";
import {useParams} from "react-router-dom";

export function NoteNew() {
    return (<NoteForm
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={"note"}
        autoSaveKey={"autosave.note.new"}
        submitText={"Create"}
        title={`New note - Notegraf`}
    />);
}

export function NoteAppend() {
    let {noteID} = useParams();

    return (<NoteForm
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={`note/${noteID}/next`}
        autoSaveKey={`autosave.note.${noteID}.append`}
        submitText={"Append"}
        title={`Append note ${noteID} - Notegraf`}
    />);
}

export function NoteBranch() {
    let {noteID} = useParams();

    return (<NoteForm
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={`note/${noteID}/branch`}
        autoSaveKey={`autosave.note.${noteID}.branch`}
        submitText={"Branch"}
        title={`Add branch ${noteID} - Notegraf`}
    />);
}
