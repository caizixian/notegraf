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
                        <Route path={"new"}>
                            <Route index element={<pages.NoteNewSessions/>}/>
                            <Route path={":sessionTs"} element={<pages.NoteNew/>}/>
                        </Route>
                        <Route path=":noteID">
                            <Route index element={<pages.NoteSequence/>}/>
                            <Route path={"edit"}>
                                <Route index element={<pages.NoteEditSessions/>}/>
                                <Route path={":sessionTs"} element={<pages.NoteEdit/>}/>
                            </Route>
                            <Route path={"branch"}>
                                <Route index element={<pages.NoteBranchSessions/>}/>
                                <Route path={":sessionTs"} element={<pages.NoteBranch/>}/>
                            </Route>
                            <Route path={"append"}>
                                <Route index element={<pages.NoteAppendSessions/>}/>
                                <Route path={":sessionTs"} element={<pages.NoteAppend/>}/>
                            </Route>
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
