import * as React from "react";
import {useEffect, useState} from "react";
import {getKeysByPrefix, getStorageValue, renderTitle, showAgo} from "../utils";
import {Link, useNavigate} from "react-router-dom";
import * as types from "../types";
import {TrashIcon} from "@heroicons/react/24/outline";

type FormSessionsProps = {
    keyPrefix: string,
    title: string
}

export function FormSessions(props: FormSessionsProps) {
    const navigate = useNavigate();
    const [keys, setKeys] = useState<string[]>([]);
    const [notes, setNotes] = useState<types.Note[]>([]);

    function loadSessions() {
        let existingKeys = getKeysByPrefix(props.keyPrefix);
        existingKeys.sort();
        let notes = existingKeys.map(key => getStorageValue(key, null));
        if (existingKeys.length == 0) {
            navigate(`./${Date.now()}`);
        } else {
            setKeys(existingKeys);
            setNotes(notes);
        }
    }

    useEffect(() => {
        document.title = props.title;
        loadSessions();
    }, []);
    return (<div>
        Choose a session:
        <div>
            {keys.map((key, idx) => {
                let parts = key.split('.');
                let ts = parts[parts.length - 1];
                let date = new Date(parseInt(ts));
                let note = notes[idx];
                let title = note ? renderTitle(note.title) : (<span></span>);
                return (<div key={key} className={"my-0.5"}>
                    <Link to={`./${ts}`} className={"underline"}>{title} ({showAgo(date)})</Link>
                    <button
                        onClick={() => {
                            if (window.confirm("Are you sure you want to delete this session?")) {
                                localStorage.removeItem(key);
                                loadSessions();
                            }
                        }}
                        className={"ng-button ng-button-danger inline ml-2"}
                        title={"Delete"}>
                        <TrashIcon className={"h-4 w-4"}/>
                    </button>
                </div>);
            })}
            <div className={"my-0.5"}><Link to={`./${Date.now()}`} className={"underline"}>New session</Link></div>
        </div>
    </div>);
}