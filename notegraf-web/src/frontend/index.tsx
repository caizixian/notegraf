import * as React from "react";
import {createRoot} from "react-dom/client";
import {BrowserRouter, Route, Routes} from "react-router-dom";
import * as pages from "./pages";
import "./app.css";

const container = document.getElementById('app') as HTMLInputElement;
const root = createRoot(container);
root.render(
    <React.StrictMode>
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<pages.App/>}>
                    <Route path="note" element={<pages.NoteTop/>}>
                        <Route index element={<pages.SearchResults/>}/>
                        <Route path={"new"} element={<pages.NoteNew/>}/>
                        <Route path=":anchorNoteID">
                            <Route index element={<pages.NoteSequence/>}/>
                            <Route path="edit" element={<pages.NoteEdit/>}/>
                            <Route path="revision">
                                <Route index element={<pages.NoteRevisions/>}/>
                                <Route path=":revision" element={<pages.NoteRevision/>}/>
                            </Route>
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
