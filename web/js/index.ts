/*import {Button, SysbadgeApp} from "../../target/wasm32-unknown-unknown/release/pkg/sysbadge_web";

const app = new SysbadgeApp({});
//app.draw();

app.register_buttons();


function animationFrame() {
    app.draw();
    //window.requestAnimationFrame(animationFrame);
}

//window.requestAnimationFrame(animationFrame);*/
import("../../target/wasm32-unknown-unknown/release/pkg/sysbadge_web").catch(console.error);