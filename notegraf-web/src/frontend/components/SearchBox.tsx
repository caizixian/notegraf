import * as React from "react";
import {SearchIcon} from "@heroicons/react/outline";
import {createSearchParams, useNavigate, useSearchParams} from "react-router-dom";

export default function SearchBox() {
    const navigate = useNavigate();
    const onKeyDown: React.KeyboardEventHandler<HTMLInputElement> = (event) => {
        if (event.key === "Enter") {
            navigate({
                pathname: "/note",
                search: createSearchParams({
                    query: event.target.value
                }).toString()
            });
        }
    }
    let [searchParams, _setSearchParams] = useSearchParams();
    let query = searchParams.get("query");
    return (
        <div className={"flex items-center w-full"}>
            <SearchIcon className={"h-6 w-6 shrink-0"}/>
            <input type="search"
                   className={"form-input bg-transparent border border-neutral-700 dark:border-neutral-300 h-8 min-w-0"}
                   onKeyDown={onKeyDown}
                   defaultValue={query ? query : undefined}/>
        </div>
    )
}