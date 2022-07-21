import {NavLink} from "react-router-dom";
import SearchBox from "./SearchBox";
import {DocumentAddIcon, LogoutIcon} from "@heroicons/react/outline";
import * as React from "react";

export function NavBar() {
    // @ts-ignore
    let sso_signout_url = document.head.querySelector('meta[name="notegraf-sso-signout"]')!.content;
    return (
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
                {sso_signout_url && <a href={sso_signout_url}>
                    <button className={"ng-button ng-button-primary"}>
                        <LogoutIcon className={"h-6 w-6"}/>
                    </button>
                </a>}
            </div>
        </nav>
    );
}