import * as React from "react";
import {useForm} from "react-hook-form";
import {useNavigate} from "react-router-dom";
import {incrementCounter, useLocalStorage} from "../utils/autosave";
import {postNote} from "../api";

export function isValidJSON(s: string): boolean {
    try {
        JSON.parse(s);
    } catch (e) {
        return false;
    }
    return true;
}

export function NewNoteForm() {
    const {register, watch, setValue, handleSubmit} = useForm();
    const navigate = useNavigate();
    useLocalStorage("autosave.note.new", watch, setValue, {
        title: "",
        note_inner: "",
        metadata_tags: "",
        metadata_custom_metadata: "{}"
    }, 5000);

    const onSubmit = async (data: any) => {
        try {
            let res = await postNote(data);
            localStorage.removeItem("autosave.note.new");
            incrementCounter();
            navigate(`/note/${res["Specific"][0]}`);
        } catch (e: any) {
            console.error('Error:', e.toString());
        }
    }

    return (
        <form className={"p-2 flex flex-col flex-1"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"title"}>Title</label>
                <input className={"form-input bg-transparent w-full"} id={"title"} placeholder={"Title"}
                       type={"text"} {...register("title")} />
                <input type="submit" className={"ng-button bg-sky-500"} value={"Create"}/>
            </div>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"tags"}>Tags</label>
                <input className={"form-input bg-transparent w-full"} id={"tags"} placeholder={"comma separated"}
                       type={"text"} {...register("metadata_tags")} />
            </div>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"custom_metadata"}>Custom metadata</label>
                <input className={"form-input bg-transparent w-full"} id={"custom_metadata"} placeholder={"{}"}
                       type={"text"} {...register("metadata_custom_metadata", {validate: isValidJSON})} />
            </div>
            <div className={"flex-1 flex"}>
                <textarea required={true} autoFocus={true} id={"note_inner"}
                          className={"form-textarea bg-transparent flex-1"} {...register("note_inner")}></textarea>
            </div>
        </form>
    );
}