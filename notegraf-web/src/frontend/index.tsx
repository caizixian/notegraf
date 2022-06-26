import * as React from "react";
import {createRoot} from "react-dom/client";
import {NoteSequence} from "./note_sequence";
import {BrowserRouter, NavLink, Outlet, Route, Routes} from "react-router-dom";
import "./markdown.tsx";
import {NewNoteForm} from "./new_note";

function App() {
    return (
        <div className={"min-h-screen bg-white dark:bg-slate-800 dark:text-white flex flex-col"}>
            <nav className={"flex p-1 w-full bg-gray-500"}>
                <NavLink to={"/note"}>New Note</NavLink>
            </nav>
            <Outlet/>
        </div>
    );
}

function Notes() {
    return (
        <Outlet/>
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
                        <Route index element={<NewNoteForm/>}/>
                        <Route path=":anchorNoteID" element={<NoteSequence/>}/>
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
