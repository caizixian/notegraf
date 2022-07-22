import * as React from "react";
import {useEffect, useState} from "react";
import {getKeysByPrefix, showAgo} from "../utils";
import {Link, useNavigate} from "react-router-dom";

type FormSessionsProps = {
    keyPrefix: string,
    title: string
}

export function FormSessions(props: FormSessionsProps) {
    const navigate = useNavigate();
    const [keys, setKeys] = useState<string[]>([]);

    useEffect(() => {
        document.title = props.title;
        const existingKeys = getKeysByPrefix(props.keyPrefix);
        if (existingKeys.length == 0) {
            navigate(`./${Date.now()}`);
        } else {
            setKeys(existingKeys);
        }
    }, []);
    return (<div>
        Choose a session:
        <ul>
            {keys.map(key => {
                let parts = key.split('.');
                let ts = parts[parts.length - 1];
                let date = new Date(parseInt(ts));
                return (<li key={key}><Link to={`./${ts}`} className={"underline"}>{showAgo(date)}</Link></li>);
            })}
            <li><Link to={`./${Date.now()}`} className={"underline"}>New session</Link></li>
        </ul>
    </div>);
}