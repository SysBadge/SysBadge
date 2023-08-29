import {Button, SysbadgeApp} from "../../target/wasm32-unknown-unknown/release/pkg/sysbadge_web";

export const app = new SysbadgeApp({});

app.press_button(Button.B);
app.draw();
