import {TagIcon} from "@heroicons/react/outline";
import {Link} from "react-router-dom";
import * as React from "react";

type TagsProps = {
    tags: string[],
    disableLink: boolean
}

export function Tags(props: TagsProps) {
    let tags = props.tags;
    tags.sort();
    return (<div className={"flex gap-1 flex-wrap"}>{props.tags.map(tag =>
        <div key={tag} className={"flex items-center rounded border border-neutral-500"}>
            <TagIcon className={"h-[1em] w-[1em] inline"}/>
            {props.disableLink ? <p className={"select-none"}>{tag}</p> :
                <Link to={"/note?" + new URLSearchParams({query: `#${tag}`})} className={"select-none"}>{tag}</Link>}
        </div>
    )}</div>);
}
