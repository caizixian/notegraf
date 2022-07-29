import * as React from "react";
import {useEffect, useState} from "react";
import {getTags} from "../api";
import {Tags} from "../components/Tags";

export function TagList() {
    const [tags, setTags] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    async function fetchTags() {
        try {
            const tags = await getTags();
            setError(null);
            setTags(tags);
            setIsLoaded(true);
            document.title = `Tags - Notegraf`;
        } catch (e) {
            setError(e);
            setIsLoaded(true);
        }
    }

    useEffect(() => {
        fetchTags();
    }, []);

    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        console.log(error);
        return (<div>{error.toString()}</div>);
    }

    return (<div className={"p-2"}>
        <h1 className={"text-4xl mb-2"}>Tags</h1>
        <Tags tags={tags} disableLink={false}/>
    </div>);
}
