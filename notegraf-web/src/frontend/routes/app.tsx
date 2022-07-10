import {NavLink, Outlet} from "react-router-dom";
import * as React from "react";

export function App() {
    return (
        <div className={"min-h-screen bg-white dark:bg-slate-800 dark:text-white flex flex-col"}>
            <nav className={"flex p-1 w-full bg-gray-500"}>
                <NavLink to={"/note/new"}>New Note</NavLink>
            </nav>
            <Outlet/>
        </div>
    );
}