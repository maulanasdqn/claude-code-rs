import { mount } from "svelte";
import App from "./app.svelte";
import "./app.css";

export default mount(App, { target: document.getElementById("app") });
