import {useForm} from "react-hook-form";
import {useNavigate, useParams} from "react-router-dom";
import {incrementCounter, useLocalStorage} from "../utils";
import {postNote} from "../api";
import * as React from "react";
import {useEffect, useState} from "react";
import {RenderMarkdown} from "./Note";
import {CodeBracketIcon, DocumentTextIcon} from "@heroicons/react/24/outline";

type NoteFormContent = {
    title: string,
    note_inner: string,
    metadata_tags: string,
    metadata_custom_metadata: string
}

type NoteFormProps = {
    defaultValue: NoteFormContent,
    endpoint: string,
    autoSaveKeyPrefix: string,
    submitText: string,
    title: string
}

function isValidJSON(s: string): boolean {
    try {
        JSON.parse(s);
    } catch (e) {
        return false;
    }
    return true;
}


export function NoteFormSession(props: NoteFormProps) {
    const {register, watch, setValue, handleSubmit, getValues} = useForm();
    const navigate = useNavigate();
    const [error, setError] = useState<any>(null);
    const [preview, setPreview] = useState(false);
    const {sessionTs} = useParams();
    const autoSaveKey = `${props.autoSaveKeyPrefix}.${sessionTs}`;
    useLocalStorage(autoSaveKey, watch, setValue, props.defaultValue, 5000);

    const onSubmit = async (data: any) => {
        try {
            let dataCloned = structuredClone(data); // Avoid mutating the original data
            dataCloned.note_inner = dataCloned.note_inner.replace(`${window.origin}/note/`, "notegraf:/note/");
            let res = await postNote(props.endpoint, dataCloned);
            localStorage.removeItem(autoSaveKey);
            incrementCounter();
            navigate(`/note/${res["Specific"][0]}`);
        } catch (e: any) {
            setError(e);
        }
    }

    useEffect(() => {
        document.title = props.title;
    }, [props.title]);

    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (
        <form className={"p-2 flex flex-col flex-1"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"title"}>Title</label>
                <input className={"form-input bg-transparent w-full"} id={"title"} placeholder={"Title"}
                       spellCheck={true} type={"text"} {...register("title")} />
                <input type="submit" className={"ng-button ng-button-primary"} value={props.submitText}/>
            </div>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"tags"}>Tags</label>
                <input className={"form-input bg-transparent w-full"} id={"tags"} placeholder={"comma separated"}
                       spellCheck={true} type={"text"} {...register("metadata_tags")} />
                <button className={"ng-button ng-button-primary"}
                        title={preview ? "Show source" : "Preview"}
                        onClick={(e) => {
                            e.preventDefault();
                            setPreview(!preview);
                        }}>
                    {preview ?
                        (<CodeBracketIcon className={"h-6 w-6"}/>) :
                        (<DocumentTextIcon className={"h-6 w-6"}/>)
                    }
                </button>
            </div>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"custom_metadata"}>Custom metadata</label>
                <input className={"form-input bg-transparent w-full"} id={"custom_metadata"} placeholder={"{}"}
                       spellCheck={true}
                       type={"text"} {...register("metadata_custom_metadata", {validate: isValidJSON})} />
            </div>
            <div className={"flex-1 flex justify-center"}>
                <textarea required={true} autoFocus={true} id={"note_inner"}
                          className={"form-textarea bg-transparent flex-1 font-mono"}
                          hidden={preview}
                          spellCheck={true}
                          {...register("note_inner")}></textarea>
                {preview && <RenderMarkdown note_inner={getValues("note_inner") || ""}/>}
            </div>
        </form>
    );
}
