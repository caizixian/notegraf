import * as React from "react";
import {createRoot} from "react-dom/client";
import {BrowserRouter, Route, Routes} from "react-router-dom";
import {NoteSequence} from "./routes/note_sequence";
import {NewNoteForm} from "./routes/new_note";
import {EditNoteForm} from "./routes/edit_note";
import {App} from "./routes/app";
import {Notes} from "./routes/notes";
import "./utils/markdown.tsx";
import "./app.css";

const container = document.getElementById('app') as HTMLInputElement;
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<App/>}>
                    <Route path="note" element={<Notes/>}>
                        <Route index element={<NewNoteForm/>}/>
                        <Route path=":anchorNoteID">
                            <Route index element={<NoteSequence/>}/>
                            <Route path="edit" element={<EditNoteForm/>}/>
                        </Route>
                    </Route>
                </Route>
                <Route
                    path="*"
                    element={
                        <p>Invalid path</p>
                    }
                />
            </Routes>
        </BrowserRouter>
    </React.StrictMode>
);
