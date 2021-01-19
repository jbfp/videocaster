<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import * as server from "../server";

    const separator = "__sep"; // replaced at compile time

    let currentDir: string | null = null;
    let selectedFile: string | null = null;
    let entries: {
        name: string;
        type: "dir" | "file";
        url: string;
        onClick(): void;
    }[] = [];

    const dispatch = createEventDispatcher();

    onMount(loadDir);

    async function loadDir() {
        const dir = await server.loadDirectoryAsync(currentDir);
        currentDir = dir.realPath;
        entries = dir.items.map(({ isDir, name }) => ({
            name,
            type: isDir ? "dir" : "file",
            url: `#${name}`,

            onClick() {
                const path = `${currentDir}${separator}${name}`;

                if (isDir) {
                    currentDir = path;
                    loadDir();
                } else {
                    selectedFile = path;
                }
            },
        }));
    }

    function next() {
        dispatch("filePicked", selectedFile);
    }
</script>

<h2>Select Video File</h2>

<input type="text" bind:value={currentDir} on:change={loadDir} />

<ul>
    {#each entries as entry}
        <li class="file-list-item" data-type={entry.type}>
            <a href={entry.url} on:click={entry.onClick}>{entry.name}</a>
        </li>
    {/each}
</ul>

{#if selectedFile}
    <div>
        <div>
            You have selected: <code>{selectedFile}</code>
        </div>

        <div>
            <button on:click={next}>Next</button>
        </div>
    </div>
{/if}

<style>
    button {
        margin-top: 1em;
    }

    ul {
        padding-left: 1em;
    }

    .file-list-item[data-type="dir"] {
        list-style: url(data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJDYXBhXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4Ig0KCSB2aWV3Qm94PSIwIDAgNDA4IDQwOCIgc3R5bGU9ImVuYWJsZS1iYWNrZ3JvdW5kOm5ldyAwIDAgNDA4IDQwODsiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0zNzIsODguNjYxSDIwNi4zMmwtMzMtMzkuMjRjLTAuOTg1LTEuMTg0LTIuNDYxLTEuODQ4LTQtMS44SDM2Yy0xOS45NTYsMC4xOTgtMzYuMDIzLDE2LjQ0My0zNiwzNi40djI0MA0KCQkJYy0wLjAwMSwxOS45NDEsMTYuMDYsMzYuMTYzLDM2LDM2LjM2aDMzNmMxOS45NC0wLjE5NywzNi4wMDEtMTYuNDE5LDM2LTM2LjM2di0xOTlDNDA4LjAwMSwxMDUuMDgsMzkxLjk0LDg4Ljg1OSwzNzIsODguNjYxeiIvPg0KCTwvZz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjwvc3ZnPg0K);
    }

    .file-list-item[data-type="file"] {
        list-style: url(data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJDYXBhXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4Ig0KCSB2aWV3Qm94PSIwIDAgNDc3Ljg2NyA0NzcuODY3IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCA0NzcuODY3IDQ3Ny44Njc7IiB4bWw6c3BhY2U9InByZXNlcnZlIj4NCjxnPg0KCTxnPg0KCQk8cGF0aCBkPSJNNDIxLjY0OSw5MC4zMTdMMzM2LjMxNiw0Ljk4M2MtMS41ODktMS41OTMtMy40ODEtMi44NTItNS41NjQtMy43MDNjLTIuMDU5LTAuODQxLTQuMjYxLTEuMjc2LTYuNDg1LTEuMjhIMTAyLjQNCgkJCUM3NC4xMjMsMCw1MS4yLDIyLjkyMyw1MS4yLDUxLjJ2Mzc1LjQ2N2MwLDI4LjI3NywyMi45MjMsNTEuMiw1MS4yLDUxLjJoMjczLjA2N2MyOC4yNzcsMCw1MS4yLTIyLjkyMyw1MS4yLTUxLjJWMTAyLjQNCgkJCUM0MjYuNjQzLDk3Ljg3LDQyNC44NDEsOTMuNTMxLDQyMS42NDksOTAuMzE3eiBNMzQxLjMzMyw1OC4yNjZsMjcuMDY4LDI3LjA2OGgtMjcuMDY4VjU4LjI2NnogTTM5Mi41MzMsNDI2LjY2Nw0KCQkJYzAsOS40MjYtNy42NDEsMTcuMDY3LTE3LjA2NywxNy4wNjdIMTAyLjRjLTkuNDI2LDAtMTcuMDY3LTcuNjQxLTE3LjA2Ny0xNy4wNjdWNTEuMmMwLTkuNDI2LDcuNjQxLTE3LjA2NywxNy4wNjctMTcuMDY3aDIwNC44DQoJCQlWMTAyLjRjMCw5LjQyNiw3LjY0MSwxNy4wNjcsMTcuMDY3LDE3LjA2N2g2OC4yNjdWNDI2LjY2N3oiLz4NCgk8L2c+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8L3N2Zz4NCg==);
    }
</style>
