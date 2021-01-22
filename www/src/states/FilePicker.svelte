<script lang="ts">
    import { createEventDispatcher, onDestroy, onMount } from "svelte";
    import type { AppResult, Directory } from "../server";
    import * as server from "../server";
    import { decode, encode } from "../encoding";
    import IconButton from "../IconButton.svelte";

    export let directory: string = "";
    export let fileName: string | null = null;

    interface Entry {
        name: string;
        path: string;
        type: "dir" | "file";
        href: string;
        onClick(): void;
    }

    let loading = false;
    let error: string | null = null;
    let entries: Entry[] = [];

    let currentDir: string = "";
    let selectedFileName: string | null = null;

    $: nextDisabled = loading || selectedFileName === null;

    $: {
        changeDir(directory);
    }

    onMount(() => window.addEventListener("popstate", onpopstate));
    onDestroy(() => window.removeEventListener("popstate", onpopstate));

    async function onpopstate(e: PopStateEvent) {
        await changeDir(e.state, false);
    }

    const dispatch = createEventDispatcher();

    async function changeDir(nextDir: string, pushState: boolean = true) {
        currentDir = nextDir;

        selectFile(null);

        let result: AppResult<Directory>;

        try {
            loading = true;
            error = null;
            result = await server.loadDirectoryAsync(currentDir);
        } finally {
            loading = false;
        }

        if (!result.success) {
            error = result.error;
            return;
        }

        if (pushState) {
            history.pushState(currentDir, "", `/${encode(currentDir)}`);
        }

        const { path, items } = result.obj;

        if (currentDir !== path) {
            currentDir = path;
            history.replaceState(currentDir, "", `/${encode(currentDir)}`);
        }

        entries = items.map(({ isDir, name, path }) => ({
            name,
            path,
            type: isDir ? "dir" : "file",
            href: isDir
                ? `/${encode(path)}`
                : `${location.pathname}/${encode(name)}`,

            onClick() {
                if (isDir) {
                    changeDir(path);
                } else {
                    selectFile(name);
                }
            },
        }));
    }

    function selectFile(s: string) {
        selectedFileName = s === selectedFileName ? null : s;
    }

    function back() {
        history.back();
    }

    function forward() {
        history.forward();
    }

    function up() {
        const split = currentDir.split("__sep");
        const path = split.slice(0, split.length - 1).join("__sep") + "__sep";
        changeDir(path);
    }

    function home() {
        changeDir("");
    }

    function change(e: Event) {
        changeDir((<HTMLInputElement>e.target).value);
    }

    function next() {
        directory = currentDir;
        fileName = selectedFileName;
        dispatch("next");
    }
</script>

<h2>Select Video File</h2>

<div class="flex flex-horizontal">
    <IconButton icon="arrow_back" title="Go back" on:click={back} />
    <IconButton icon="arrow_forward" title="Go forward" on:click={forward} />
    <IconButton icon="arrow_upward" title="Go up" on:click={up} />
    <IconButton icon="home" title="Go to Home" on:click={home} />
    <input
        class="fill"
        type="text"
        value={currentDir}
        on:change={change}
    />
</div>

{#if error}
    <div class="fill">{error}</div>
{:else}
    <ul class="fill">
        {#each entries as entry}
            <li class="file-list-item" data-type={entry.type}>
                <a
                    class={selectedFileName === entry.name ? "active" : undefined}
                    href={loading ? undefined : entry.href}
                    on:click|preventDefault={entry.onClick}
                    disabled={loading}>{entry.name}</a
                >
            </li>
        {/each}
    </ul>
{/if}

<div class="flex-horizontal">
    <button disabled={nextDisabled} on:click={next}>Next</button>

    {#if selectedFileName}
        <span>Selected video file: <code>{selectedFileName}</code></span>
    {/if}
</div>

<style>
    ul {
        margin: 0;
        padding-left: 1em;
        overflow-y: scroll;
    }

    .file-list-item[data-type="dir"] {
        list-style: url("data:image/svg+xml,%3Csvg version='1.1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 408 408'%3E%3Cpath d='M372,88.661H206.32l-33-39.24c-0.985-1.184-2.461-1.848-4-1.8H36c-19.956,0.198-36.023,16.443-36,36.4v240 c-0.001,19.941,16.06,36.163,36,36.36h336c19.94-0.197,36.001-16.419,36-36.36v-199C408.001,105.08,391.94,88.859,372,88.661z'/%3E%3C/svg%3E");
    }

    .file-list-item[data-type="file"] {
        list-style: url("data:image/svg+xml,%3Csvg version='1.1' id='Capa_1' xmlns='http://www.w3.org/2000/svg' x='0px' y='0px' viewBox='0 0 477.867 477.867'%3E%3Cpath d='M421.649,90.317L336.316,4.983c-1.589-1.593-3.481-2.852-5.564-3.703c-2.059-0.841-4.261-1.276-6.485-1.28H102.4 C74.123,0,51.2,22.923,51.2,51.2v375.467c0,28.277,22.923,51.2,51.2,51.2h273.067c28.277,0,51.2-22.923,51.2-51.2V102.4 C426.643,97.87,424.841,93.531,421.649,90.317z M341.333,58.266l27.068,27.068h-27.068V58.266z M392.533,426.667 c0,9.426-7.641,17.067-17.067,17.067H102.4c-9.426,0-17.067-7.641-17.067-17.067V51.2c0-9.426,7.641-17.067,17.067-17.067h204.8 V102.4c0,9.426,7.641,17.067,17.067,17.067h68.267V426.667z'/%3E%3C/svg%3E%0A");
    }

    a {
        border-bottom: 0 solid currentColor;
        line-height: 200%;
        padding-bottom: 1px;
        text-decoration: none;
    }

    a:hover {
        border-bottom-width: 1px;
    }

    a.active {
        border-bottom-width: 1px;
        color: -webkit-activelink;
    }

    a[disabled="true"] {
        color: currentColor;
        cursor: unset;
        opacity: 0.5;
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
