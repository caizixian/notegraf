import './app.css';
import * as React from "react";
import {Note, NoteComponent} from "./note"
import {withRouter} from "./util";

type NoteSequenceProps = {
    router: any
}

type NoteSequenceState = {
    recursiveLoad: boolean,
    isLoaded: boolean,
    notes: Note[],
    error: any
}

class NoteSequence extends React.Component<NoteSequenceProps, NoteSequenceState> {
    state: NoteSequenceState;

    private getInitialState(): NoteSequenceState {
        return {
            recursiveLoad: this.props.router.searchParams.get("recursiveLoad") === "true",
            isLoaded: false,
            notes: [],
            error: null
        };
    };

    constructor(props: NoteSequenceProps) {
        super(props);
        this.state = this.getInitialState();
        this.handleCheckbox = this.handleCheckbox.bind(this);
    }

    private readonly handleCheckbox = async (event: React.FormEvent<HTMLInputElement>) => {
        const {name, checked} = event.currentTarget;
        // @ts-ignore
        this.setState({
            [name]: checked
        });
        // @ts-ignore
        this.props.router.setSearchParams({
            [name]: checked
        });
        await this.fetchNoteSequence();
    };

    static async fetchNote(noteID: string): Promise<Note> {
        const response = await fetch(`/api/v1/note/${noteID}`);
        if (!response.ok) {
            throw new Error(response.statusText);
        }
        return response.json();
    }

    async fetchNoteSequence() {
        let notes: Note[] = [];
        try {
            let anchorNote = await NoteSequence.fetchNote(this.props.router.params.anchorNoteID);
            notes.push(anchorNote);
            if (this.state.recursiveLoad) {
                while (notes[0].prev != null) {
                    let note = await NoteSequence.fetchNote(notes[0].prev);
                    notes = [note, ...notes];
                }
                while (notes[notes.length - 1].next != null) {
                    let note = await NoteSequence.fetchNote(notes[notes.length - 1].next as string);
                    notes.push(note);
                }
            }
            this.setState({
                isLoaded: true,
                notes: notes,
                error: null
            })
        } catch (e) {
            this.setState({
                isLoaded: true,
                notes: [],
                error: e
            });
        }
    }

    async componentDidMount() {
        await this.fetchNoteSequence()
    }

    render() {
        return (<div className="note-sequence">
            <label><input type="checkbox" id="recursiveLoad" name="recursiveLoad" checked={this.state.recursiveLoad}
                          onChange={this.handleCheckbox}/>Enable
                Recursive Load</label>
            {this.state.notes.map(note =>
                <NoteComponent note={note} key={note.id} showPrevNext={!this.state.recursiveLoad}></NoteComponent>
            )}
        </div>)
    }
}

export let NoteSequenceWithRouter = withRouter(NoteSequence);
