import App from './App.svelte';
import './index.css';

window['__onGCastApiAvailable'] = (isAvailable: boolean) => {
    if (isAvailable) {
        new App({
            target: document.getElementById('app'),
            props: {},
        });
    }
};
