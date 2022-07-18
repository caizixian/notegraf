import {Outlet} from "react-router-dom";
import * as React from "react";
import {NavBar} from "../components/NavBar";

export function App() {
    return (
        <div className={"min-h-screen bg-white dark:bg-slate-800 dark:text-white flex flex-col"}>
            <NavBar/>
            <Outlet/>
        </div>
    );
}