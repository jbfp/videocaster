/// <reference types="chromecast-caf-sender" />
import * as server from "./server";
import App from "./App.svelte";
import "./index.css";

window.addEventListener("unload", server.shutdown);

window.__onGCastApiAvailable = (isAvailable) => {
    const context = cast.framework.CastContext.getInstance();

    context.setOptions({
        receiverApplicationId:
            chrome.cast.media.DEFAULT_MEDIA_RECEIVER_APP_ID,
        autoJoinPolicy: chrome.cast.AutoJoinPolicy.TAB_AND_ORIGIN_SCOPED,
    });

    window["__isGCastApiAvailable"] = isAvailable;
};

new App({
    target: document.body,
    props: {},
});
