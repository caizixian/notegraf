import * as React from "react";
import {NoteFormSession} from "../components/NoteFormSession";
import {useParams} from "react-router-dom";
import {FormSessions} from "../components/FormSessions";

export function NoteNew() {
    return (<NoteFormSession
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={"note"}
        autoSaveKeyPrefix={"autosave.note.new"}
        submitText={"Create"}
        title={`New note - Notegraf`}
    />);
}

export function NoteNewSessions() {
    return (<FormSessions keyPrefix={`autosave.note.new`}
                          title={`Choose a session: new note - Notegraf`}/>);
}

export function NoteAppend() {
    let {noteID} = useParams();

    return (<NoteFormSession
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={`note/${noteID}/next`}
        autoSaveKeyPrefix={`autosave.note.${noteID}.append`}
        submitText={"Append"}
        title={`Append note ${noteID} - Notegraf`}
    />);
}

export function NoteAppendSessions() {
    let {noteID} = useParams();
    return (<FormSessions keyPrefix={`autosave.note.${noteID}.append`}
                          title={`Choose a session: append note ${noteID} - Notegraf`}/>);
}

export function NoteBranch() {
    let {noteID} = useParams();

    return (<NoteFormSession
        defaultValue={{
            title: "",
            note_inner: "",
            metadata_tags: "",
            metadata_custom_metadata: "{}"
        }}
        endpoint={`note/${noteID}/branch`}
        autoSaveKeyPrefix={`autosave.note.${noteID}.branch`}
        submitText={"Branch"}
        title={`Add branch ${noteID} - Notegraf`}
    />);
}

export function NoteBranchSessions() {
    let {noteID} = useParams();
    return (<FormSessions keyPrefix={`autosave.note.${noteID}.branch`}
                          title={`Choose a session: add branch ${noteID} - Notegraf`}/>);
}