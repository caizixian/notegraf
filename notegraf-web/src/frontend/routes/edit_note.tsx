import {useNavigate, useParams} from "react-router-dom";
import * as React from "react";
import {useEffect, useState} from "react";
import {useForm} from "react-hook-form";
import {incrementCounter, useLocalStorage} from "../utils/autosave";
import {getNote, updateNote} from "../api";
import {Note} from "../note";
import {isValidJSON} from "./new_note";

type EditNoteInnerProps = {
    note: Note
}

function EditNoteInner(props: EditNoteInnerProps) {
    const {register, watch, setValue, handleSubmit} = useForm();
    const navigate = useNavigate();
    useLocalStorage(`autosave.note.edit.${props.note.id}`, watch, setValue, {
        title: props.note.title,
        note_inner: props.note.note_inner,
        metadata_tags: props.note.metadata.tags.join(", "),
        metadata_custom_metadata: JSON.stringify(props.note.metadata.custom_metadata)
    }, 5000);

    const onSubmit = async (data: any) => {
        try {
            let res = await updateNote(props.note.id, data);
            localStorage.removeItem(`autosave.note.edit.${props.note.id}`);
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
                <input type="submit" className={"ng-button bg-sky-500"} value={"Update"}/>
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

export function EditNoteForm() {
    let {anchorNoteID} = useParams();
    const [note, setNote] = useState<any>(null);
    const [error, setError] = useState<any>(null);
    const [isLoaded, setIsLoaded] = useState(false);

    useEffect(() => {
        async function fetchNote() {
            try {
                const note = await getNote(anchorNoteID as string);
                setNote(note);
                setIsLoaded(true);
            } catch (e) {
                setError(e);
                setIsLoaded(true);
            }
        }

        fetchNote();
    }, [anchorNoteID]);


    if (!isLoaded) {
        return (<div>Loading...</div>);
    }
    if (error) {
        return (<div>{error.toString()}</div>);
    }

    return (<EditNoteInner note={note}/>);
}