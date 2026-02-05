import { mount } from "svelte";
import ShowcaseApp from "./ShowcaseApp.svelte";
import "./app.css";

const app = mount(ShowcaseApp, {
  target: document.getElementById("app")!,
});

export default app;
