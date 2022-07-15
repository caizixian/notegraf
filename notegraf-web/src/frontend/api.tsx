import {Note} from "./note";

type NoteLocator = {
    Specific: string[]
}

export async function getNote(noteID: string): Promise<Note> {
    const response = await fetch(`/api/v1/note/${noteID}`);
    if (!response.ok) {
        throw new Error(response.statusText);
    }
    return response.json();
}

export async function getNoteSpecific(noteID: string, revision: string): Promise<Note> {
    const response = await fetch(`/api/v1/note/${noteID}/revision/${revision}`);
    if (!response.ok) {
        throw new Error(response.statusText);
    }
    return response.json();
}

export async function getNoteRevisions(noteID: string): Promise<Note[]> {
    const response = await fetch(`/api/v1/note/${noteID}/revision`);
    if (!response.ok) {
        throw new Error(response.statusText);
    }
    return response.json();
}

export async function searchNotes(query: string): Promise<Note[]> {
    const response = await fetch("/api/v1/note?" + new URLSearchParams({query: query}));
    if (!response.ok) {
        throw new Error(response.statusText);
    }
    return response.json();
}

export async function deleteNote(noteID: string) {
    const response = await fetch(`/api/v1/note/${noteID}`, {
        method: "DELETE"
    });
    if (!response.ok) {
        throw new Error(response.statusText);
    }
}

export async function postNote(data: any): Promise<NoteLocator> {
    let response = await fetch('/api/v1/note', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
    });
    if (!response.ok) {
        console.error('Error:', response.statusText);
    }
    return response.json();
}

export async function updateNote(noteID: string, data: any): Promise<NoteLocator> {
    let response = await fetch(`/api/v1/note/${noteID}/revision`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
    });
    if (!response.ok) {
        console.error('Error:', response.statusText);
    }
    return response.json();
}