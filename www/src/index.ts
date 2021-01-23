import * as server from "./server";
import App from './App.svelte';
import './index.css';

window.addEventListener("unload", server.shutdown);

window['__onGCastApiAvailable'] = (isAvailable: boolean) => {
    if (isAvailable) {
        new App({
            target: document.body,
            props: {},
        });
    }
};
