import * as React from "react";
import {NoteForm} from "../components/NoteForm";


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

    />);
}