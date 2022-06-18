import * as React from "react";
import {createRoot} from "react-dom/client";
import {NoteSequence} from "./note_sequence";

const container = document.getElementById('app') as HTMLInputElement;
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <NoteSequence anchorNoteID="note-1"></NoteSequence>
    </React.StrictMode>
);
