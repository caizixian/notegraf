import * as React from "react";
import {useForm} from "react-hook-form";
import {useNavigate} from "react-router-dom";

export function NewNoteForm() {
    const {register, handleSubmit} = useForm();
    const navigate = useNavigate();

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
                navigate(`./${data["Specific"][0]}`)
            })
            .catch((error) => {
                console.error('Error:', error);
            });
    }

    return (
        <form className={"border p-1 flex flex-col h-screen"} onSubmit={handleSubmit(onSubmit)}>
            <div className={"flex"}>
                <label htmlFor={"title"}>Title</label>
                <input className={"form-input dark:bg-slate-800 w-full"} id={"title"} placeholder={"Title"}
                       type={"text"} {...register("title")} />
                <input type="submit" className={"rounded-md bg-sky-500 block"} value={"Create"}/>
            </div>
            <div className={"flex h-full"}>
                <textarea required={true} autoFocus={true} id={"note_inner"}
                          className={"form-textarea dark:bg-slate-800 w-full h-full"} {...register("note_inner")}></textarea>
            </div>
        </form>
    );
}