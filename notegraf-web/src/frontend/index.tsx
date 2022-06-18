import * as React from "react";
import {createRoot} from "react-dom/client";
import {NoteSequence} from "./note_sequence";
import {BrowserRouter, Outlet, Route, Routes} from "react-router-dom";
import "./markdown.tsx";

function App() {
    return (
        <div className={"h-screen bg-white dark:bg-slate-800 dark:text-white"}>
            <h1>Notegraf</h1>
            <Outlet/>
        </div>
    );
}

function Notes() {
    return (
        <div>
            <h1>Note Sequence View</h1>
            <Outlet/>
        </div>
    );
}

const container = document.getElementById('app') as HTMLInputElement;
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<App/>}>
                    <Route path="note" element={<Notes/>}>
                        <Route
                            index
                            element={
                                <p>Probably a search bar here</p>
                            }
                        />
                        <Route path=":anchorNoteID" element={<NoteSequence/>}></Route>
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
