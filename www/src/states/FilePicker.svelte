<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import type { Directory } from "../server";
    import * as server from "../server";

    interface Entry {
        name: string;
        path: string;
        type: "dir" | "file";
        href: string;
        onClick(): void;
    }

    let loading = false;
    let currentDir: string | null = null;
    let selectedFile: string | null = null;
    let entries: Entry[] = [];

    $: fileName = selectedFile?.replace(currentDir, "")?.replace(/^\\/, "");
    $: nextDisabled = loading || selectedFile === null;

    const dispatch = createEventDispatcher();

    onMount(() => {
        const hash = decodeURIComponent(location.hash.slice(1));

        if (hash) {
            const separator = "__sep"; // replaced at compile time
            const split = hash.split(separator);
            const len = split.length;

            if (len > 0 && split[len - 1].includes(".")) {
                currentDir = split.slice(0, len - 1).join(separator);
                selectedFile = hash;
            } else {
                currentDir = hash;
            }
        }

        loadDir();
    });

    async function loadDir() {
        let dir: Directory;

        try {
            loading = true;
            dir = await server.loadDirectoryAsync(currentDir);
        } finally {
            loading = false;
        }

        currentDir = dir.realPath;
        history.replaceState("", "", `#${currentDir}`);
        entries = dir.items.map(({ isDir, name, path }) => ({
            name,
            path,
            type: isDir ? "dir" : "file",
            href: `#${path}`,

            onClick() {
                if (isDir) {
                    currentDir = path;
                    selectedFile = null;
                    loadDir();
                } else {
                    selectedFile = path;
                    history.replaceState("", "", this.href);
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
            <a
                href={loading ? undefined : entry.href}
                on:click|preventDefault={entry.onClick}
                disabled={loading}>{entry.name}</a
            >
        </li>
    {/each}
</ul>

<div>
    <button disabled={nextDisabled} on:click={next}>Next</button>

    {#if fileName}
        Selected video file: <code>{fileName}</code>
    {/if}
</div>

<style>
    button {
        margin-top: 1em;
    }

    ul {
        padding-left: 1em;
    }

    .file-list-item[data-type="dir"] {
        list-style: url("data:image/svg+xml,%3Csvg version='1.1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 408 408'%3E%3Cpath d='M372,88.661H206.32l-33-39.24c-0.985-1.184-2.461-1.848-4-1.8H36c-19.956,0.198-36.023,16.443-36,36.4v240 c-0.001,19.941,16.06,36.163,36,36.36h336c19.94-0.197,36.001-16.419,36-36.36v-199C408.001,105.08,391.94,88.859,372,88.661z'/%3E%3C/svg%3E");
    }

    .file-list-item[data-type="file"] {
        list-style: url("data:image/svg+xml,%3Csvg version='1.1' id='Capa_1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 477.867 477.867'%3E%3Cpath d='M421.649,90.317L336.316,4.983c-1.589-1.593-3.481-2.852-5.564-3.703c-2.059-0.841-4.261-1.276-6.485-1.28H102.4 C74.123,0,51.2,22.923,51.2,51.2v375.467c0,28.277,22.923,51.2,51.2,51.2h273.067c28.277,0,51.2-22.923,51.2-51.2V102.4 C426.643,97.87,424.841,93.531,421.649,90.317z M341.333,58.266l27.068,27.068h-27.068V58.266z M392.533,426.667 c0,9.426-7.641,17.067-17.067,17.067H102.4c-9.426,0-17.067-7.641-17.067-17.067V51.2c0-9.426,7.641-17.067,17.067-17.067h204.8 V102.4c0,9.426,7.641,17.067,17.067,17.067h68.267V426.667z'/%3E%3C/svg%3E%0A");
    }

    a[disabled="true"] {
        color: currentColor;
        cursor: unset;
        opacity: 0.5;
        text-decoration: underline;
    }

    @media (prefers-color-scheme: dark) {
        .file-list-item[data-type="dir"] {
            list-style: url("data:image/svg+xml,%3Csvg version='1.1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 408 408'%3E%3Cpath style='fill: white' d='M372,88.661H206.32l-33-39.24c-0.985-1.184-2.461-1.848-4-1.8H36c-19.956,0.198-36.023,16.443-36,36.4v240 c-0.001,19.941,16.06,36.163,36,36.36h336c19.94-0.197,36.001-16.419,36-36.36v-199C408.001,105.08,391.94,88.859,372,88.661z'/%3E%3C/svg%3E");
        }

        .file-list-item[data-type="file"] {
            list-style: url("data:image/svg+xml,%3Csvg version='1.1' id='Capa_1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 477.867 477.867'%3E%3Cpath style='fill: white' d='M421.649,90.317L336.316,4.983c-1.589-1.593-3.481-2.852-5.564-3.703c-2.059-0.841-4.261-1.276-6.485-1.28H102.4 C74.123,0,51.2,22.923,51.2,51.2v375.467c0,28.277,22.923,51.2,51.2,51.2h273.067c28.277,0,51.2-22.923,51.2-51.2V102.4 C426.643,97.87,424.841,93.531,421.649,90.317z M341.333,58.266l27.068,27.068h-27.068V58.266z M392.533,426.667 c0,9.426-7.641,17.067-17.067,17.067H102.4c-9.426,0-17.067-7.641-17.067-17.067V51.2c0-9.426,7.641-17.067,17.067-17.067h204.8 V102.4c0,9.426,7.641,17.067,17.067,17.067h68.267V426.667z'/%3E%3C/svg%3E%0A");
        }
    }
</style>
