import * as React from "react";
import {createRoot} from "react-dom/client";
import './app.css';

const container = document.getElementById('app') as HTMLInputElement;
const root = createRoot(container);
root.render(
    <React.StrictMode></React.StrictMode>
);
