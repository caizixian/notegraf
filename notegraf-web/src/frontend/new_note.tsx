import * as React from "react";
import {useForm} from "react-hook-form";
import {useNavigate} from "react-router-dom";
import {useLocalStorage} from "./autosave";

export function NewNoteForm() {
    const {register, watch, setValue, handleSubmit} = useForm();
    const navigate = useNavigate();
    useLocalStorage("autosave.note.new", watch, setValue, {title: "", note_inner: ""}, 5000);

    const onSubmit = (data: any) => {
        fetch('/api/v1/note', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        })
            .then(response => response.json())
            .then(data => {
                localStorage.removeItem("autosave.note.new");
                navigate(`./${data["Specific"][0]}`)
            })
            .catch((error) => {
                console.error('Error:', error);
            });
    }

    return (
        <form className={"p-2 flex flex-col flex-1"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"flex gap-2 m-1 items-center"}>
                <label htmlFor={"title"}>Title</label>
                <input className={"form-input dark:bg-slate-800 w-full"} id={"title"} placeholder={"Title"}
                       type={"text"} {...register("title")} />
                <input type="submit" className={"rounded-md bg-sky-500 block p-1"} value={"Create"}/>
            </div>
            <div className={"flex-1"}>
                <textarea required={true} autoFocus={true} id={"note_inner"}
                          className={"form-textarea dark:bg-slate-800 w-full h-full"} {...register("note_inner")}></textarea>
            </div>
        </form>
    );
}