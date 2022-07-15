import {NavLink, Outlet} from "react-router-dom";
import * as React from "react";
import SearchBox from "../components/SearchBox";
import {DocumentAddIcon} from "@heroicons/react/outline";

export function App() {
    return (
        <div className={"min-h-screen bg-white dark:bg-slate-800 dark:text-white flex flex-col"}>
            <nav className={"flex p-1 w-full bg-gray-500 items-center gap-1"}>
                <NavLink to={"/note"}>Recent</NavLink>
                <div className={"ml-auto flex gap-1 items-center min-w-0"}>
                    <div className={"min-w-0"}>
                        <SearchBox/>
                    </div>
                    <NavLink to={"/note/new"}>
                        <button className={"ng-button ng-button-primary"}>
                            <DocumentAddIcon className={"h-6 w-6"}/>
                        </button>
                    </NavLink>
                </div>
            </nav>
            <Outlet/>
        </div>
    );
}